use acvm::{acir::circuit::Circuit, compiler::AcirTransformationMap};
use iter_extended::{try_vecmap, vecmap};
use nargo::artifacts::contract::PreprocessedContractFunction;
use nargo::artifacts::debug::DebugArtifact;
use nargo::artifacts::program::PreprocessedProgram;
use nargo::package::Package;
use nargo::prepare_package;
use nargo::{artifacts::contract::PreprocessedContract, NargoError};
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_driver::{
    compile_contracts, compile_main, CompileOptions, CompiledContract, CompiledProgram,
    ErrorsAndWarnings, Warnings,
};
use noirc_errors::debug_info::DebugInfo;
use noirc_frontend::graph::CrateName;
use noirc_frontend::hir::Context;

use clap::Args;

use crate::backends::Backend;
use crate::errors::{CliError, CompileError};

use super::fs::program::{
    save_contract_to_file, save_debug_artifact_to_file, save_program_to_file,
};
use super::NargoConfig;

// TODO(#1388): pull this from backend.
const BACKEND_IDENTIFIER: &str = "acvm-backend-barretenberg";

/// Compile the program and its secret execution trace into ACIR format
#[derive(Debug, Clone, Args)]
pub(crate) struct CompileCommand {
    /// Include Proving and Verification keys in the build artifacts.
    #[arg(long)]
    include_keys: bool,

    /// Output debug files
    #[arg(long, hide = true)]
    output_debug: bool,

    /// The name of the package to compile
    #[clap(long, conflicts_with = "workspace")]
    package: Option<CrateName>,

    /// Compile all packages in the workspace
    #[clap(long, conflicts_with = "package")]
    workspace: bool,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run(
    backend: &Backend,
    args: CompileCommand,
    config: NargoConfig,
) -> Result<(), CliError> {
    let toml_path = get_package_manifest(&config.program_dir)?;
    let default_selection =
        if args.workspace { PackageSelection::All } else { PackageSelection::DefaultOrAll };
    let selection = args.package.map_or(default_selection, PackageSelection::Selected);
    let workspace = resolve_workspace_from_toml(&toml_path, selection)?;
    let circuit_dir = workspace.target_directory_path();

    for package in &workspace {
        let (mut context, crate_id) = prepare_package(package);
        // If `contract` package type, we're compiling every function in a 'contract' rather than just 'main'.
        if package.is_contract() {
            let result = compile_contracts(&mut context, crate_id, &args.compile_options);
            let contracts = report_errors(result, &context, args.compile_options.deny_warnings)?;
            let optimized_contracts =
                try_vecmap(contracts, |contract| optimize_contract(backend, contract))?;

            // TODO(#1389): I wonder if it is incorrect for nargo-core to know anything about contracts.
            // As can be seen here, It seems like a leaky abstraction where ContractFunctions (essentially CompiledPrograms)
            // are compiled via nargo-core and then the PreprocessedContract is constructed here.
            // This is due to EACH function needing it's own CRS, PKey, and VKey from the backend.
            let preprocessed_contracts: Vec<(PreprocessedContract, Vec<DebugInfo>)> =
                vecmap(optimized_contracts, |contract| {
                    let preprocess_result = vecmap(contract.functions, |func| {
                        (
                            PreprocessedContractFunction {
                                name: func.name,
                                function_type: func.function_type,
                                is_internal: func.is_internal,
                                abi: func.abi,

                                bytecode: func.bytecode,
                            },
                            func.debug,
                        )
                    });

                    let (preprocessed_contract_functions, debug_infos): (Vec<_>, Vec<_>) =
                        preprocess_result.into_iter().unzip();

                    (
                        PreprocessedContract {
                            name: contract.name,
                            backend: String::from(BACKEND_IDENTIFIER),
                            functions: preprocessed_contract_functions,
                        },
                        debug_infos,
                    )
                });
            for (contract, debug_infos) in preprocessed_contracts {
                save_contract_to_file(
                    &contract,
                    &format!("{}-{}", package.name, contract.name),
                    &circuit_dir,
                );

                if args.output_debug {
                    let debug_artifact = DebugArtifact::new(debug_infos, &context);
                    save_debug_artifact_to_file(
                        &debug_artifact,
                        &format!("{}-{}", package.name, contract.name),
                        &circuit_dir,
                    );
                }
            }
        } else {
            let (context, program) = compile_package(backend, package, &args.compile_options)?;

            let preprocessed_program = PreprocessedProgram {
                backend: String::from(BACKEND_IDENTIFIER),
                abi: program.abi,
                bytecode: program.circuit,
            };

            save_program_to_file(&preprocessed_program, &package.name, &circuit_dir);

            if args.output_debug {
                let debug_artifact = DebugArtifact::new(vec![program.debug], &context);
                let circuit_name: String = (&package.name).into();
                save_debug_artifact_to_file(&debug_artifact, &circuit_name, &circuit_dir);
            }
        }
    }

    Ok(())
}

pub(crate) fn compile_package(
    backend: &Backend,
    package: &Package,
    compile_options: &CompileOptions,
) -> Result<(Context, CompiledProgram), CompileError> {
    if package.is_library() {
        return Err(CompileError::LibraryCrate(package.name.clone()));
    }

    let (mut context, crate_id) = prepare_package(package);
    let result = compile_main(&mut context, crate_id, compile_options);
    let mut program = report_errors(result, &context, compile_options.deny_warnings)?;
    // Apply backend specific optimizations.
    let (optimized_circuit, location_map) = optimize_circuit(backend, program.circuit)
        .expect("Backend does not support an opcode that is in the IR");
    // TODO(#2110): Why does this set `program.circuit` to `optimized_circuit` instead of the function taking ownership
    // and requiring we use `optimized_circuit` everywhere after
    program.circuit = optimized_circuit;
    program.debug.update_acir(location_map);

    Ok((context, program))
}

pub(super) fn optimize_circuit(
    backend: &Backend,
    circuit: Circuit,
) -> Result<(Circuit, AcirTransformationMap), CliError> {
    let result = acvm::compiler::compile(circuit, backend.np_language(), |opcode| {
        backend.supports_opcode(opcode)
    })
    .map_err(|_| NargoError::CompilationError)?;

    Ok(result)
}

pub(super) fn optimize_contract(
    backend: &Backend,
    contract: CompiledContract,
) -> Result<CompiledContract, CliError> {
    let functions = try_vecmap(contract.functions, |mut func| {
        let (optimized_bytecode, location_map) = optimize_circuit(backend, func.bytecode)?;
        func.bytecode = optimized_bytecode;
        func.debug.update_acir(location_map);
        Ok::<_, CliError>(func)
    })?;

    Ok(CompiledContract { functions, ..contract })
}

/// Helper function for reporting any errors in a Result<(T, Warnings), ErrorsAndWarnings>
/// structure that is commonly used as a return result in this file.
pub(crate) fn report_errors<T>(
    result: Result<(T, Warnings), ErrorsAndWarnings>,
    context: &Context,
    deny_warnings: bool,
) -> Result<T, CompileError> {
    let (t, warnings) = result.map_err(|errors| {
        noirc_errors::reporter::report_all(&context.file_manager, &errors, deny_warnings)
    })?;

    noirc_errors::reporter::report_all(&context.file_manager, &warnings, deny_warnings);
    Ok(t)
}
