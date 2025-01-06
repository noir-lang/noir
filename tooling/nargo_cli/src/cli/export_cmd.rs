use nargo::errors::CompileError;
use nargo::ops::report_errors;
use noirc_errors::FileDiagnostic;
use noirc_frontend::hir::ParsedFiles;
use rayon::prelude::*;

use fm::FileManager;
use iter_extended::try_vecmap;
use nargo::package::Package;
use nargo::prepare_package;
use nargo::workspace::Workspace;
use nargo::{insert_all_files_for_workspace_into_file_manager, parse_all};
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml};
use noirc_driver::{
    compile_no_check, CompileOptions, CompiledProgram, NOIR_ARTIFACT_VERSION_STRING,
};

use clap::Args;

use crate::errors::CliError;

use super::check_cmd::check_crate_and_report_errors;

use super::fs::program::save_program_to_file;
use super::{NargoConfig, PackageOptions};

/// Exports functions marked with #[export] attribute
#[derive(Debug, Clone, Args)]
pub(crate) struct ExportCommand {
    #[clap(flatten)]
    pub(super) package_options: PackageOptions,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run(args: ExportCommand, config: NargoConfig) -> Result<(), CliError> {
    let toml_path = get_package_manifest(&config.program_dir)?;
    let selection = args.package_options.package_selection();
    let workspace = resolve_workspace_from_toml(
        &toml_path,
        selection,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_owned()),
    )?;

    let mut workspace_file_manager = workspace.new_file_manager();
    insert_all_files_for_workspace_into_file_manager(&workspace, &mut workspace_file_manager);
    let parsed_files = parse_all(&workspace_file_manager);

    let library_packages: Vec<_> =
        workspace.into_iter().filter(|package| package.is_library()).collect();

    library_packages
        .par_iter()
        .map(|package| {
            compile_exported_functions(
                &workspace_file_manager,
                &parsed_files,
                &workspace,
                package,
                &args.compile_options,
            )
        })
        .collect()
}

fn compile_exported_functions(
    file_manager: &FileManager,
    parsed_files: &ParsedFiles,
    workspace: &Workspace,
    package: &Package,
    compile_options: &CompileOptions,
) -> Result<(), CliError> {
    let (mut context, crate_id) = prepare_package(file_manager, parsed_files, package);
    check_crate_and_report_errors(&mut context, crate_id, compile_options)?;

    let exported_functions = context.get_all_exported_functions_in_crate(&crate_id);

    let exported_programs = try_vecmap(
        exported_functions,
        |(function_name, function_id)| -> Result<(String, CompiledProgram), CompileError> {
            // TODO: We should to refactor how to deal with compilation errors to avoid this.
            let program = compile_no_check(&mut context, compile_options, function_id, None, false)
                .map_err(|error| vec![FileDiagnostic::from(error)]);

            let program = report_errors(
                program.map(|program| (program, Vec::new())),
                file_manager,
                compile_options.deny_warnings,
                compile_options.silence_warnings,
            )?;

            Ok((function_name, program))
        },
    )?;

    let export_dir = workspace.export_directory_path();
    for (function_name, program) in exported_programs {
        save_program_to_file(&program.into(), &function_name.parse().unwrap(), &export_dir);
    }
    Ok(())
}
