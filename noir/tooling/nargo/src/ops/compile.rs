use acvm::ExpressionWidth;
use fm::FileManager;
use noirc_driver::{CompilationResult, CompileOptions, CompiledContract, CompiledProgram};

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
    workspace: &Workspace,
    binary_packages: &[Package],
    contract_packages: &[Package],
    expression_width: ExpressionWidth,
    compile_options: &CompileOptions,
) -> Result<(Vec<CompiledProgram>, Vec<CompiledContract>), CompileError> {
    // Compile all of the packages in parallel.
    let program_results: Vec<(FileManager, CompilationResult<CompiledProgram>)> = binary_packages
        .par_iter()
        .map(|package| compile_program(workspace, package, compile_options, expression_width))
        .collect();
    let contract_results: Vec<(FileManager, CompilationResult<CompiledContract>)> =
        contract_packages
            .par_iter()
            .map(|package| compile_contract(package, compile_options, expression_width))
            .collect();

    // Report any warnings/errors which were encountered during compilation.
    let compiled_programs: Vec<CompiledProgram> = program_results
        .into_iter()
        .map(|(file_manager, compilation_result)| {
            report_errors(
                compilation_result,
                &file_manager,
                compile_options.deny_warnings,
                compile_options.silence_warnings,
            )
        })
        .collect::<Result<_, _>>()?;
    let compiled_contracts: Vec<CompiledContract> = contract_results
        .into_iter()
        .map(|(file_manager, compilation_result)| {
            report_errors(
                compilation_result,
                &file_manager,
                compile_options.deny_warnings,
                compile_options.silence_warnings,
            )
        })
        .collect::<Result<_, _>>()?;

    Ok((compiled_programs, compiled_contracts))
}

pub fn compile_program(
    workspace: &Workspace,
    package: &Package,
    compile_options: &CompileOptions,
    expression_width: ExpressionWidth,
) -> (FileManager, CompilationResult<CompiledProgram>) {
    let (mut context, crate_id) = prepare_package(package);

    let program_artifact_path = workspace.package_build_path(package);
    let mut debug_artifact_path = program_artifact_path.clone();
    debug_artifact_path.set_file_name(format!("debug_{}.json", package.name));

    let (program, warnings) =
        match noirc_driver::compile_main(&mut context, crate_id, compile_options, None, true) {
            Ok(program_and_warnings) => program_and_warnings,
            Err(errors) => {
                return (context.file_manager, Err(errors));
            }
        };

    // Apply backend specific optimizations.
    let optimized_program = crate::ops::optimize_program(program, expression_width);

    (context.file_manager, Ok((optimized_program, warnings)))
}

fn compile_contract(
    package: &Package,
    compile_options: &CompileOptions,
    expression_width: ExpressionWidth,
) -> (FileManager, CompilationResult<CompiledContract>) {
    let (mut context, crate_id) = prepare_package(package);
    let (contract, warnings) =
        match noirc_driver::compile_contract(&mut context, crate_id, compile_options) {
            Ok(contracts_and_warnings) => contracts_and_warnings,
            Err(errors) => {
                return (context.file_manager, Err(errors));
            }
        };

    let optimized_contract = crate::ops::optimize_contract(contract, expression_width);

    (context.file_manager, Ok((optimized_contract, warnings)))
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
