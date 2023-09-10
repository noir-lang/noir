use std::path::Path;

use acvm::acir::circuit::Opcode;
use acvm::Language;
use fm::FileManager;
use iter_extended::{try_vecmap, vecmap};
use nargo::artifacts::contract::PreprocessedContract;
use nargo::artifacts::contract::PreprocessedContractFunction;
use nargo::artifacts::debug::DebugArtifact;
use nargo::artifacts::program::PreprocessedProgram;
use nargo::package::Package;
use nargo::prepare_package;
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_driver::{
    compile_main, CompileOptions, CompiledContract, CompiledProgram, ErrorsAndWarnings, Warnings,
};
use noirc_errors::debug_info::DebugInfo;
use noirc_frontend::graph::CrateName;

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
    println!("foo");
    dbg!("foo1");
    let toml_path = get_package_manifest(&config.program_dir)?;
    let default_selection =
        if args.workspace { PackageSelection::All } else { PackageSelection::DefaultOrAll };
    let selection = args.package.map_or(default_selection, PackageSelection::Selected);
    let workspace = resolve_workspace_from_toml(&toml_path, selection)?;
    let circuit_dir = workspace.target_directory_path();

    let (np_language, is_opcode_supported) = backend.get_backend_info()?;
    for package in &workspace {
        // If `contract` package type, we're compiling every function in a 'contract' rather than just 'main'.
        if package.is_contract() {
            let (file_manager, contracts) = compile_contracts(
                package,
                &args.compile_options,
                np_language,
                &is_opcode_supported,
            )?;
            save_contracts(&file_manager, contracts, package, &circuit_dir, args.output_debug);
        } else {
            let (file_manager, program) =
                compile_package(package, &args.compile_options, np_language, &is_opcode_supported)?;
            save_program(&file_manager, program, package, &circuit_dir, args.output_debug);
        }
    }

    Ok(())
}

pub(crate) fn compile_package(
    package: &Package,
    compile_options: &CompileOptions,
    np_language: Language,
    is_opcode_supported: &impl Fn(&Opcode) -> bool,
) -> Result<(FileManager, CompiledProgram), CliError> {
    if package.is_library() {
        return Err(CompileError::LibraryCrate(package.name.clone()).into());
    }

    let (mut context, crate_id) = prepare_package(package);
    let result = compile_main(&mut context, crate_id, compile_options);
    let program = report_errors(result, &context.file_manager, compile_options.deny_warnings)?;

    // Apply backend specific optimizations.
    let optimized_program =
        nargo::ops::optimize_program(program, np_language, &is_opcode_supported)
            .expect("Backend does not support an opcode that is in the IR");

    Ok((context.file_manager, optimized_program))
}

pub(crate) fn compile_contracts(
    package: &Package,
    compile_options: &CompileOptions,
    np_language: Language,
    is_opcode_supported: &impl Fn(&Opcode) -> bool,
) -> Result<(FileManager, Vec<CompiledContract>), CliError> {
    let (mut context, crate_id) = prepare_package(package);
    let result = noirc_driver::compile_contracts(&mut context, crate_id, compile_options);
    let contracts = report_errors(result, &context.file_manager, compile_options.deny_warnings)?;

    let optimized_contracts = try_vecmap(contracts, |contract| {
        nargo::ops::optimize_contract(contract, np_language, &is_opcode_supported)
    })?;
    Ok((context.file_manager, optimized_contracts))
}

fn save_program(
    file_manager: &FileManager,
    program: CompiledProgram,
    package: &Package,
    circuit_dir: &Path,
    output_debug: bool,
) {
    let preprocessed_program = PreprocessedProgram {
        backend: String::from(BACKEND_IDENTIFIER),
        abi: program.abi,
        bytecode: program.circuit,
    };

    save_program_to_file(&preprocessed_program, &package.name, circuit_dir);

    if output_debug {
        let debug_artifact = DebugArtifact::new(vec![program.debug], file_manager);
        let circuit_name: String = (&package.name).into();
        save_debug_artifact_to_file(&debug_artifact, &circuit_name, circuit_dir);
    }
}

fn save_contracts(
    file_manager: &FileManager,
    contracts: Vec<CompiledContract>,
    package: &Package,
    circuit_dir: &Path,
    output_debug: bool,
) {
    // TODO(#1389): I wonder if it is incorrect for nargo-core to know anything about contracts.
    // As can be seen here, It seems like a leaky abstraction where ContractFunctions (essentially CompiledPrograms)
    // are compiled via nargo-core and then the PreprocessedContract is constructed here.
    // This is due to EACH function needing it's own CRS, PKey, and VKey from the backend.
    let preprocessed_contracts: Vec<(PreprocessedContract, Vec<DebugInfo>)> =
        vecmap(contracts, |contract| {
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
            circuit_dir,
        );

        if output_debug {
            let debug_artifact = DebugArtifact::new(debug_infos, file_manager);
            save_debug_artifact_to_file(
                &debug_artifact,
                &format!("{}-{}", package.name, contract.name),
                circuit_dir,
            );
        }
    }
}

/// Helper function for reporting any errors in a Result<(T, Warnings), ErrorsAndWarnings>
/// structure that is commonly used as a return result in this file.
pub(crate) fn report_errors<T>(
    result: Result<(T, Warnings), ErrorsAndWarnings>,
    file_manager: &FileManager,
    deny_warnings: bool,
) -> Result<T, CompileError> {
    let (t, warnings) = result.map_err(|errors| {
        noirc_errors::reporter::report_all(file_manager.as_file_map(), &errors, deny_warnings)
    })?;

    noirc_errors::reporter::report_all(file_manager.as_file_map(), &warnings, deny_warnings);
    Ok(t)
}
