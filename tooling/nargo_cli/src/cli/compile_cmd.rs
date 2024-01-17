use std::path::Path;

use acvm::ExpressionWidth;

use fm::FileManager;
use nargo::artifacts::program::ProgramArtifact;
use nargo::errors::CompileError;
use nargo::insert_all_files_for_workspace_into_file_manager;
use nargo::package::Package;
use nargo::prepare_package;
use nargo::workspace::Workspace;
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_driver::file_manager_with_stdlib;
use noirc_driver::NOIR_ARTIFACT_VERSION_STRING;
use noirc_driver::{CompilationResult, CompileOptions, CompiledContract, CompiledProgram};

use noirc_frontend::graph::CrateName;

use clap::Args;

use crate::backends::Backend;
use crate::errors::CliError;

use super::fs::program::only_acir;
use super::fs::program::{read_program_from_file, save_contract_to_file, save_program_to_file};
use super::NargoConfig;
use rayon::prelude::*;

/// Compile the program and its secret execution trace into ACIR format
#[derive(Debug, Clone, Args)]
pub(crate) struct CompileCommand {
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

    let workspace = resolve_workspace_from_toml(
        &toml_path,
        selection,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_owned()),
    )?;
    let circuit_dir = workspace.target_directory_path();

    let mut workspace_file_manager = file_manager_with_stdlib(&workspace.root_dir);
    insert_all_files_for_workspace_into_file_manager(&workspace, &mut workspace_file_manager);

    let (binary_packages, contract_packages): (Vec<_>, Vec<_>) = workspace
        .into_iter()
        .filter(|package| !package.is_library())
        .cloned()
        .partition(|package| package.is_binary());

    let expression_width = backend.get_backend_info_or_default();
    let (_, compiled_contracts) = compile_workspace(
        &workspace_file_manager,
        &workspace,
        &binary_packages,
        &contract_packages,
        expression_width,
        &args.compile_options,
    )?;

    // Save build artifacts to disk.
    for (package, contract) in contract_packages.into_iter().zip(compiled_contracts) {
        save_contract(contract, &package, &circuit_dir);
    }

    Ok(())
}

pub(super) fn compile_workspace(
    file_manager: &FileManager,
    workspace: &Workspace,
    binary_packages: &[Package],
    contract_packages: &[Package],
    expression_width: ExpressionWidth,
    compile_options: &CompileOptions,
) -> Result<(Vec<CompiledProgram>, Vec<CompiledContract>), CliError> {
    // Compile all of the packages in parallel.
    let program_results: Vec<CompilationResult<CompiledProgram>> = binary_packages
        .par_iter()
        .map(|package| {
            compile_program(file_manager, workspace, package, compile_options, expression_width)
        })
        .collect();
    let contract_results: Vec<CompilationResult<CompiledContract>> = contract_packages
        .par_iter()
        .map(|package| compile_contract(file_manager, package, compile_options, expression_width))
        .collect();

    // Report any warnings/errors which were encountered during compilation.
    let compiled_programs: Vec<CompiledProgram> = program_results
        .into_iter()
        .map(|compilation_result| {
            report_errors(
                compilation_result,
                file_manager,
                compile_options.deny_warnings,
                compile_options.silence_warnings,
            )
        })
        .collect::<Result<_, _>>()?;
    let compiled_contracts: Vec<CompiledContract> = contract_results
        .into_iter()
        .map(|compilation_result| {
            report_errors(
                compilation_result,
                file_manager,
                compile_options.deny_warnings,
                compile_options.silence_warnings,
            )
        })
        .collect::<Result<_, _>>()?;

    Ok((compiled_programs, compiled_contracts))
}

pub(crate) fn compile_bin_package(
    file_manager: &FileManager,
    workspace: &Workspace,
    package: &Package,
    compile_options: &CompileOptions,
    expression_width: ExpressionWidth,
) -> Result<CompiledProgram, CliError> {
    if package.is_library() {
        return Err(CompileError::LibraryCrate(package.name.clone()).into());
    }

    let compilation_result =
        compile_program(file_manager, workspace, package, compile_options, expression_width);

    let program = report_errors(
        compilation_result,
        file_manager,
        compile_options.deny_warnings,
        compile_options.silence_warnings,
    )?;

    Ok(program)
}

fn compile_program(
    file_manager: &FileManager,
    workspace: &Workspace,
    package: &Package,
    compile_options: &CompileOptions,
    expression_width: ExpressionWidth,
) -> CompilationResult<CompiledProgram> {
    let (mut context, crate_id) = prepare_package(file_manager, package);

    let program_artifact_path = workspace.package_build_path(package);
    let cached_program: Option<CompiledProgram> =
        read_program_from_file(program_artifact_path)
        .ok()
        .filter(|p| p.noir_version == NOIR_ARTIFACT_VERSION_STRING)
        .map(|p| p.into());

    let (program, warnings) =
        noirc_driver::compile_main(&mut context, crate_id, compile_options, cached_program)?;

    // Apply backend specific optimizations.
    let optimized_program = nargo::ops::optimize_program(program, expression_width);
    let only_acir = compile_options.only_acir;
    save_program(optimized_program.clone(), package, &workspace.target_directory_path(), only_acir);

    Ok((optimized_program, warnings))
}

fn compile_contract(
    file_manager: &FileManager,
    package: &Package,
    compile_options: &CompileOptions,
    expression_width: ExpressionWidth,
) -> CompilationResult<CompiledContract> {
    let (mut context, crate_id) = prepare_package(file_manager, package);
    let (contract, warnings) =
        match noirc_driver::compile_contract(&mut context, crate_id, compile_options) {
            Ok(contracts_and_warnings) => contracts_and_warnings,
            Err(errors) => {
                return Err(errors);
            }
        };

    let optimized_contract = nargo::ops::optimize_contract(contract, expression_width);

    Ok((optimized_contract, warnings))
}

pub(super) fn save_program(
    program: CompiledProgram,
    package: &Package,
    circuit_dir: &Path,
    only_acir_opt: bool,
) {
    let program_artifact = ProgramArtifact::from(program.clone());
    if only_acir_opt {
        only_acir(&program_artifact, circuit_dir);
    } else {
        save_program_to_file(&program_artifact, &package.name, circuit_dir);
    }
}

fn save_contract(contract: CompiledContract, package: &Package, circuit_dir: &Path) {
    let contract_name = contract.name.clone();
    save_contract_to_file(
        &contract.into(),
        &format!("{}-{}", package.name, contract_name),
        circuit_dir,
    );
}

/// Helper function for reporting any errors in a `CompilationResult<T>`
/// structure that is commonly used as a return result in this file.
pub(crate) fn report_errors<T>(
    result: CompilationResult<T>,
    file_manager: &FileManager,
    deny_warnings: bool,
    silence_warnings: bool,
) -> Result<T, CompileError> {
    let (t, warnings) = result.map_err(|errors| {
        noirc_errors::reporter::report_all(
            file_manager.as_file_map(),
            &errors,
            deny_warnings,
            silence_warnings,
        )
    })?;

    noirc_errors::reporter::report_all(
        file_manager.as_file_map(),
        &warnings,
        deny_warnings,
        silence_warnings,
    );

    Ok(t)
}
