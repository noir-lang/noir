use fm::FileManager;
use noirc_driver::{
    link_to_debug_crate, CompilationResult, CompileOptions, CompiledContract, CompiledProgram,
};
use noirc_frontend::debug::DebugInstrumenter;
use noirc_frontend::hir::ParsedFiles;

use crate::errors::CompileError;
use crate::prepare_package;
use crate::{package::Package, workspace::Workspace};

use rayon::prelude::*;

/// Compiles workspace.
///
/// # Errors
///
/// This function will return an error if there are any compilations errors reported.
pub fn compile_workspace(
    file_manager: &FileManager,
    parsed_files: &ParsedFiles,
    workspace: &Workspace,
    compile_options: &CompileOptions,
) -> CompilationResult<(Vec<CompiledProgram>, Vec<CompiledContract>)> {
    let (binary_packages, contract_packages): (Vec<_>, Vec<_>) = workspace
        .into_iter()
        .filter(|package| !package.is_library())
        .cloned()
        .partition(|package| package.is_binary());

    // Compile all of the packages in parallel.
    let program_results: Vec<CompilationResult<CompiledProgram>> = binary_packages
        .par_iter()
        .map(|package| {
            compile_program(file_manager, parsed_files, workspace, package, compile_options, None)
        })
        .collect();
    let contract_results: Vec<CompilationResult<CompiledContract>> = contract_packages
        .par_iter()
        .map(|package| compile_contract(file_manager, parsed_files, package, compile_options))
        .collect();

    // Collate any warnings/errors which were encountered during compilation.
    let compiled_programs = collect_errors(program_results);
    let compiled_contracts = collect_errors(contract_results);

    match (compiled_programs, compiled_contracts) {
        (Ok((programs, program_warnings)), Ok((contracts, contract_warnings))) => {
            let warnings = [program_warnings, contract_warnings].concat();
            Ok(((programs, contracts), warnings))
        }
        (Err(program_errors), Err(contract_errors)) => {
            Err([program_errors, contract_errors].concat())
        }
        (Err(errors), _) | (_, Err(errors)) => Err(errors),
    }
}

pub fn compile_program(
    file_manager: &FileManager,
    parsed_files: &ParsedFiles,
    workspace: &Workspace,
    package: &Package,
    compile_options: &CompileOptions,
    cached_program: Option<CompiledProgram>,
) -> CompilationResult<CompiledProgram> {
    compile_program_with_debug_instrumenter(
        file_manager,
        parsed_files,
        workspace,
        package,
        compile_options,
        cached_program,
        DebugInstrumenter::default(),
    )
}

#[tracing::instrument(level = "trace", name = "compile_program" skip_all, fields(package = package.name.to_string()))]
pub fn compile_program_with_debug_instrumenter(
    file_manager: &FileManager,
    parsed_files: &ParsedFiles,
    workspace: &Workspace,
    package: &Package,
    compile_options: &CompileOptions,
    cached_program: Option<CompiledProgram>,
    debug_instrumenter: DebugInstrumenter,
) -> CompilationResult<CompiledProgram> {
    let (mut context, crate_id) = prepare_package(file_manager, parsed_files, package);
    link_to_debug_crate(&mut context, crate_id);
    context.debug_instrumenter = debug_instrumenter;
    context.package_build_path = workspace.package_build_path(package);

    noirc_driver::compile_main(&mut context, crate_id, compile_options, cached_program)
}

#[tracing::instrument(level = "trace", skip_all, fields(package_name = package.name.to_string()))]
pub fn compile_contract(
    file_manager: &FileManager,
    parsed_files: &ParsedFiles,
    package: &Package,
    compile_options: &CompileOptions,
) -> CompilationResult<CompiledContract> {
    let (mut context, crate_id) = prepare_package(file_manager, parsed_files, package);
    noirc_driver::compile_contract(&mut context, crate_id, compile_options)
}

/// Constructs a single `CompilationResult` for a collection of `CompilationResult`s, merging the set of warnings/errors.
pub fn collect_errors<T>(results: Vec<CompilationResult<T>>) -> CompilationResult<Vec<T>> {
    let mut artifacts = Vec::new();
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    for result in results {
        match result {
            Ok((new_artifact, new_warnings)) => {
                artifacts.push(new_artifact);
                warnings.extend(new_warnings);
            }
            Err(new_errors) => errors.extend(new_errors),
        }
    }

    if errors.is_empty() {
        Ok((artifacts, warnings))
    } else {
        Err(errors)
    }
}

pub fn report_errors<T>(
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
