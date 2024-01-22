use acvm::ExpressionWidth;
use fm::FileManager;
use noirc_driver::{CompilationResult, CompileOptions, CompiledContract, CompiledProgram};
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
    expression_width: ExpressionWidth,
    compile_options: &CompileOptions,
) -> Result<(Vec<CompiledProgram>, Vec<CompiledContract>), CompileError> {
    let (binary_packages, contract_packages): (Vec<_>, Vec<_>) = workspace
        .into_iter()
        .filter(|package| !package.is_library())
        .cloned()
        .partition(|package| package.is_binary());

    // Compile all of the packages in parallel.
    let program_results: Vec<CompilationResult<CompiledProgram>> = binary_packages
        .par_iter()
        .map(|package| {
            compile_program(
                file_manager,
                parsed_files,
                package,
                compile_options,
                expression_width,
                None,
            )
        })
        .collect();
    let contract_results: Vec<CompilationResult<CompiledContract>> = contract_packages
        .par_iter()
        .map(|package| {
            compile_contract(file_manager, parsed_files, package, compile_options, expression_width)
        })
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

pub fn compile_program(
    file_manager: &FileManager,
    parsed_files: &ParsedFiles,
    package: &Package,
    compile_options: &CompileOptions,
    expression_width: ExpressionWidth,
    cached_program: Option<CompiledProgram>,
) -> CompilationResult<CompiledProgram> {
    let (mut context, crate_id) = prepare_package(file_manager, parsed_files, package);

    let (program, warnings) =
        noirc_driver::compile_main(&mut context, crate_id, compile_options, cached_program)?;

    // Apply backend specific optimizations.
    let optimized_program = crate::ops::optimize_program(program, expression_width);

    Ok((optimized_program, warnings))
}

pub fn compile_contract(
    file_manager: &FileManager,
    parsed_files: &ParsedFiles,
    package: &Package,
    compile_options: &CompileOptions,
    expression_width: ExpressionWidth,
) -> CompilationResult<CompiledContract> {
    let (mut context, crate_id) = prepare_package(file_manager, parsed_files, package);
    let (contract, warnings) =
        noirc_driver::compile_contract(&mut context, crate_id, compile_options)?;

    let optimized_contract = crate::ops::optimize_contract(contract, expression_width);

    Ok((optimized_contract, warnings))
}

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
