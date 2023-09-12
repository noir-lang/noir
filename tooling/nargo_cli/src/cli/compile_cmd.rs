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
use noirc_driver::{CompilationResult, CompileOptions, CompiledContract, CompiledProgram};
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
            let (file_manager, compilation_result) = compile_contracts(
                package,
                &args.compile_options,
                np_language,
                &is_opcode_supported,
            );
            let contracts_with_debug_artifacts = report_errors(
                compilation_result,
                &file_manager,
                args.compile_options.deny_warnings,
            )?;

            save_contracts(
                contracts_with_debug_artifacts,
                package,
                &circuit_dir,
                args.output_debug,
            );
        } else {
            let (file_manager, compilation_result) =
                compile_program(package, &args.compile_options, np_language, &is_opcode_supported);

            let (program, debug_artifact) = report_errors(
                compilation_result,
                &file_manager,
                args.compile_options.deny_warnings,
            )?;
            save_program(debug_artifact, program, package, &circuit_dir, args.output_debug);
        }
    }

    Ok(())
}

pub(crate) fn compile_bin_package(
    package: &Package,
    compile_options: &CompileOptions,
    np_language: Language,
    is_opcode_supported: &impl Fn(&Opcode) -> bool,
) -> Result<(CompiledProgram, DebugArtifact), CliError> {
    if package.is_library() {
        return Err(CompileError::LibraryCrate(package.name.clone()).into());
    }

    let (file_manager, compilation_result) =
        compile_program(package, compile_options, np_language, &is_opcode_supported);

    let (program, debug_artifact) =
        report_errors(compilation_result, &file_manager, compile_options.deny_warnings)?;

    Ok((program, debug_artifact))
}

pub(crate) fn compile_contract_package(
    package: &Package,
    compile_options: &CompileOptions,
    np_language: Language,
    is_opcode_supported: &impl Fn(&Opcode) -> bool,
) -> Result<Vec<(CompiledContract, DebugArtifact)>, CliError> {
    let (file_manager, compilation_result) =
        compile_contracts(package, compile_options, np_language, &is_opcode_supported);
    let contracts_with_debug_artifacts =
        report_errors(compilation_result, &file_manager, compile_options.deny_warnings)?;
    Ok(contracts_with_debug_artifacts)
}

fn compile_program(
    package: &Package,
    compile_options: &CompileOptions,
    np_language: Language,
    is_opcode_supported: &impl Fn(&Opcode) -> bool,
) -> (FileManager, CompilationResult<(CompiledProgram, DebugArtifact)>) {
    let (mut context, crate_id) = prepare_package(package);

    let (program, warnings) =
        match noirc_driver::compile_main(&mut context, crate_id, compile_options) {
            Ok(program_and_warnings) => program_and_warnings,
            Err(errors) => {
                return (context.file_manager, Err(errors));
            }
        };

    // Apply backend specific optimizations.
    let optimized_program =
        nargo::ops::optimize_program(program, np_language, &is_opcode_supported)
            .expect("Backend does not support an opcode that is in the IR");

    let debug_artifact =
        DebugArtifact::new(vec![optimized_program.debug.clone()], &context.file_manager);

    (context.file_manager, Ok(((optimized_program, debug_artifact), warnings)))
}

fn compile_contracts(
    package: &Package,
    compile_options: &CompileOptions,
    np_language: Language,
    is_opcode_supported: &impl Fn(&Opcode) -> bool,
) -> (FileManager, CompilationResult<Vec<(CompiledContract, DebugArtifact)>>) {
    let (mut context, crate_id) = prepare_package(package);
    let (contracts, warnings) =
        match noirc_driver::compile_contracts(&mut context, crate_id, compile_options) {
            Ok(contracts_and_warnings) => contracts_and_warnings,
            Err(errors) => {
                return (context.file_manager, Err(errors));
            }
        };

    let optimized_contracts = try_vecmap(contracts, |contract| {
        nargo::ops::optimize_contract(contract, np_language, &is_opcode_supported)
    })
    .expect("Backend does not support an opcode that is in the IR");

    let contracts_with_debug_artifacts = vecmap(optimized_contracts, |contract| {
        let debug_infos = vecmap(&contract.functions, |func| func.debug.clone());
        let debug_artifact = DebugArtifact::new(debug_infos, &context.file_manager);

        (contract, debug_artifact)
    });

    (context.file_manager, Ok((contracts_with_debug_artifacts, warnings)))
}

fn save_program(
    debug_artifact: DebugArtifact,
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
        let circuit_name: String = (&package.name).into();
        save_debug_artifact_to_file(&debug_artifact, &circuit_name, circuit_dir);
    }
}

fn save_contracts(
    contracts: Vec<(CompiledContract, DebugArtifact)>,
    package: &Package,
    circuit_dir: &Path,
    output_debug: bool,
) {
    // TODO(#1389): I wonder if it is incorrect for nargo-core to know anything about contracts.
    // As can be seen here, It seems like a leaky abstraction where ContractFunctions (essentially CompiledPrograms)
    // are compiled via nargo-core and then the PreprocessedContract is constructed here.
    // This is due to EACH function needing it's own CRS, PKey, and VKey from the backend.
    let preprocessed_contracts: Vec<(PreprocessedContract, DebugArtifact)> =
        vecmap(contracts, |(contract, debug_artifact)| {
            let preprocessed_functions =
                vecmap(contract.functions, |func| PreprocessedContractFunction {
                    name: func.name,
                    function_type: func.function_type,
                    is_internal: func.is_internal,
                    abi: func.abi,

                    bytecode: func.bytecode,
                });

            (
                PreprocessedContract {
                    name: contract.name,
                    backend: String::from(BACKEND_IDENTIFIER),
                    functions: preprocessed_functions,
                },
                debug_artifact,
            )
        });

    for (contract, debug_artifact) in preprocessed_contracts {
        save_contract_to_file(
            &contract,
            &format!("{}-{}", package.name, contract.name),
            circuit_dir,
        );

        if output_debug {
            save_debug_artifact_to_file(
                &debug_artifact,
                &format!("{}-{}", package.name, contract.name),
                circuit_dir,
            );
        }
    }
}

/// Helper function for reporting any errors in a `CompilationResult<T>`
/// structure that is commonly used as a return result in this file.
pub(crate) fn report_errors<T>(
    result: CompilationResult<T>,
    file_manager: &FileManager,
    deny_warnings: bool,
) -> Result<T, CompileError> {
    let (t, warnings) = result.map_err(|errors| {
        noirc_errors::reporter::report_all(file_manager.as_file_map(), &errors, deny_warnings)
    })?;

    noirc_errors::reporter::report_all(file_manager.as_file_map(), &warnings, deny_warnings);
    Ok(t)
}
