use fm::FileManager;
use noirc_driver::{
    CompilationResult, CompileOptions, CompiledContract, CompiledProgram, CrateId, check_crate,
    link_to_debug_crate,
};
use noirc_frontend::debug::DebugInstrumenter;
use noirc_frontend::hir::{Context, ParsedFiles};

use crate::errors::CompileError;
use crate::prepare_package;
use crate::{package::Package, workspace::Workspace};

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

    if errors.is_empty() { Ok((artifacts, warnings)) } else { Err(errors) }
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

/// Run the lexing, parsing, name resolution, and type checking passes and report any warnings
/// and errors found.
pub fn check_crate_and_report_errors(
    context: &mut Context,
    crate_id: CrateId,
    options: &CompileOptions,
) -> Result<(), CompileError> {
    let result = check_crate(context, crate_id, options);
    report_errors(result, &context.file_manager, options.deny_warnings, options.silence_warnings)
}
