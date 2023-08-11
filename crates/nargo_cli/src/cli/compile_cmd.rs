use acvm::acir::circuit::OpcodeLabel;
use acvm::{acir::circuit::Circuit, Backend};
use iter_extended::try_vecmap;
use iter_extended::vecmap;
use nargo::artifacts::debug::DebugArtifact;
use nargo::package::Package;
use nargo::prepare_package;
use nargo::{artifacts::contract::PreprocessedContract, NargoError};
use nargo_toml::{find_package_manifest, resolve_workspace_from_toml};
use noirc_driver::{
    compile_contracts, compile_main, CompileOptions, CompiledContract, CompiledProgram,
    ErrorsAndWarnings, Warnings,
};
use noirc_errors::debug_info::DebugInfo;
use noirc_frontend::graph::CrateName;
use noirc_frontend::hir::Context;

use clap::Args;

use nargo::ops::{preprocess_contract_function, preprocess_program};

use crate::errors::{CliError, CompileError};

use super::fs::program::save_debug_artifact_to_file;
use super::fs::{
    common_reference_string::{
        read_cached_common_reference_string, update_common_reference_string,
        write_cached_common_reference_string,
    },
    program::{save_contract_to_file, save_program_to_file},
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
    #[clap(long)]
    package: Option<CrateName>,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run<B: Backend>(
    backend: &B,
    args: CompileCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {
    let toml_path = find_package_manifest(&config.program_dir)?;
    let workspace = resolve_workspace_from_toml(&toml_path, args.package)?;
    let circuit_dir = workspace.target_directory_path();

    let mut common_reference_string = read_cached_common_reference_string();

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
            let preprocessed_contracts: Result<
                Vec<(PreprocessedContract, Vec<DebugInfo>)>,
                CliError<B>,
            > = try_vecmap(optimized_contracts, |contract| {
                let preprocess_result = try_vecmap(contract.functions, |func| {
                    common_reference_string = update_common_reference_string(
                        backend,
                        &common_reference_string,
                        &func.bytecode,
                    )
                    .map_err(CliError::CommonReferenceStringError)?;

                    preprocess_contract_function(
                        backend,
                        args.include_keys,
                        &common_reference_string,
                        func,
                    )
                    .map_err(CliError::ProofSystemCompilerError)
                })?;

                let (preprocessed_contract_functions, debug_infos): (Vec<_>, Vec<_>) =
                    preprocess_result.into_iter().unzip();

                Ok((
                    PreprocessedContract {
                        name: contract.name,
                        backend: String::from(BACKEND_IDENTIFIER),
                        functions: preprocessed_contract_functions,
                    },
                    debug_infos,
                ))
            });
            for (contract, debug_infos) in preprocessed_contracts? {
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

            common_reference_string =
                update_common_reference_string(backend, &common_reference_string, &program.circuit)
                    .map_err(CliError::CommonReferenceStringError)?;

            let (preprocessed_program, debug_info) =
                preprocess_program(backend, args.include_keys, &common_reference_string, program)
                    .map_err(CliError::ProofSystemCompilerError)?;
            save_program_to_file(&preprocessed_program, &package.name, &circuit_dir);

            if args.output_debug {
                let debug_artifact = DebugArtifact::new(vec![debug_info], &context);
                let circuit_name: String = (&package.name).into();
                save_debug_artifact_to_file(&debug_artifact, &circuit_name, &circuit_dir);
            }
        }
    }

    write_cached_common_reference_string(&common_reference_string);

    Ok(())
}

pub(crate) fn compile_package<B: Backend>(
    backend: &B,
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
    let (optimized_circuit, opcode_labels) = optimize_circuit(backend, program.circuit)
        .expect("Backend does not support an opcode that is in the IR");

    // TODO(#2110): Why does this set `program.circuit` to `optimized_circuit` instead of the function taking ownership
    // and requiring we use `optimized_circuit` everywhere after
    program.circuit = optimized_circuit;
    let opcode_ids = vecmap(opcode_labels, |label| match label {
        OpcodeLabel::Unresolved => {
            unreachable!("Compiled circuit opcodes must resolve to some index")
        }
        OpcodeLabel::Resolved(index) => index as usize,
    });
    program.debug.update_acir(opcode_ids);

    Ok((context, program))
}

pub(super) fn optimize_circuit<B: Backend>(
    backend: &B,
    circuit: Circuit,
) -> Result<(Circuit, Vec<OpcodeLabel>), CliError<B>> {
    let result = acvm::compiler::compile(circuit, backend.np_language(), |opcode| {
        backend.supports_opcode(opcode)
    })
    .map_err(|_| NargoError::CompilationError)?;

    Ok(result)
}

pub(super) fn optimize_contract<B: Backend>(
    backend: &B,
    contract: CompiledContract,
) -> Result<CompiledContract, CliError<B>> {
    let functions = try_vecmap(contract.functions, |mut func| {
        let (optimized_bytecode, opcode_labels) = optimize_circuit(backend, func.bytecode)?;
        let opcode_ids = vecmap(opcode_labels, |label| match label {
            OpcodeLabel::Unresolved => {
                unreachable!("Compiled circuit opcodes must resolve to some index")
            }
            OpcodeLabel::Resolved(index) => index as usize,
        });

        func.bytecode = optimized_bytecode;
        func.debug.update_acir(opcode_ids);
        Ok::<_, CliError<B>>(func)
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
