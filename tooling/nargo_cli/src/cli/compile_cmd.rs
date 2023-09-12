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
use rayon::prelude::*;

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

    let (binary_packages, contract_packages): (Vec<_>, Vec<_>) = workspace
        .members
        .iter()
        .filter(|package| !package.is_library())
        .partition(|package| package.is_binary());

    // Compile all of the packages in parallel.
    let program_results: Vec<(FileManager, CompilationResult<CompiledProgram>)> = binary_packages
        .par_iter()
        .map(|package| {
            compile_program(package, &args.compile_options, np_language, &is_opcode_supported)
        })
        .collect();
    let contract_results: Vec<(FileManager, CompilationResult<Vec<CompiledContract>>)> =
        contract_packages
            .par_iter()
            .map(|package| {
                compile_contracts(package, &args.compile_options, np_language, &is_opcode_supported)
            })
            .collect();

    // Report any warnings/errors which were encountered during compilation.
    let compiled_programs: Vec<CompiledProgram> = program_results
        .into_iter()
        .map(|(file_manager, compilation_result)| {
            report_errors(compilation_result, &file_manager, args.compile_options.deny_warnings)
        })
        .collect::<Result<_, _>>()?;
    let compiled_contracts: Vec<Vec<CompiledContract>> = contract_results
        .into_iter()
        .map(|(file_manager, compilation_result)| {
            report_errors(compilation_result, &file_manager, args.compile_options.deny_warnings)
        })
        .collect::<Result<_, _>>()?;

    // Save build artifacts to disk.
    for (package, program) in binary_packages.into_iter().zip(compiled_programs) {
        save_program(program, package, &circuit_dir, args.output_debug);
    }
    for (package, compiled_contracts) in contract_packages.into_iter().zip(compiled_contracts) {
        save_contracts(compiled_contracts, package, &circuit_dir, args.output_debug);
    }

    Ok(())
}

pub(crate) fn compile_bin_package(
    package: &Package,
    compile_options: &CompileOptions,
    np_language: Language,
    is_opcode_supported: &impl Fn(&Opcode) -> bool,
) -> Result<CompiledProgram, CliError> {
    if package.is_library() {
        return Err(CompileError::LibraryCrate(package.name.clone()).into());
    }

    let (file_manager, compilation_result) =
        compile_program(package, compile_options, np_language, &is_opcode_supported);

    let program = report_errors(compilation_result, &file_manager, compile_options.deny_warnings)?;

    Ok(program)
}

pub(crate) fn compile_contract_package(
    package: &Package,
    compile_options: &CompileOptions,
    np_language: Language,
    is_opcode_supported: &impl Fn(&Opcode) -> bool,
) -> Result<Vec<CompiledContract>, CliError> {
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
) -> (FileManager, CompilationResult<CompiledProgram>) {
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

    (context.file_manager, Ok((optimized_program, warnings)))
}

fn compile_contracts(
    package: &Package,
    compile_options: &CompileOptions,
    np_language: Language,
    is_opcode_supported: &impl Fn(&Opcode) -> bool,
) -> (FileManager, CompilationResult<Vec<CompiledContract>>) {
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

    (context.file_manager, Ok((optimized_contracts, warnings)))
}

fn save_program(
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
        let debug_artifact = DebugArtifact {
            debug_symbols: vec![program.debug.clone()],
            file_map: program.file_map,
        };
        let circuit_name: String = (&package.name).into();
        save_debug_artifact_to_file(&debug_artifact, &circuit_name, circuit_dir);
    }
}

fn save_contracts(
    contracts: Vec<CompiledContract>,
    package: &Package,
    circuit_dir: &Path,
    output_debug: bool,
) {
    // TODO(#1389): I wonder if it is incorrect for nargo-core to know anything about contracts.
    // As can be seen here, It seems like a leaky abstraction where ContractFunctions (essentially CompiledPrograms)
    // are compiled via nargo-core and then the PreprocessedContract is constructed here.
    // This is due to EACH function needing it's own CRS, PKey, and VKey from the backend.
    let preprocessed_contracts: Vec<(PreprocessedContract, DebugArtifact)> =
        vecmap(contracts, |contract| {
            let debug_artifact = DebugArtifact {
                debug_symbols: contract
                    .functions
                    .iter()
                    .map(|function| function.debug.clone())
                    .collect(),
                file_map: contract.file_map,
            };

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
