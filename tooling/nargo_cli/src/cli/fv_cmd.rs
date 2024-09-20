use clap::Args;
use nargo::{
    insert_all_files_for_workspace_into_file_manager, ops::report_errors, parse_all,
    prepare_package,
};
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_driver::{
    file_manager_with_stdlib, link_to_debug_crate, CompileOptions, NOIR_ARTIFACT_VERSION_STRING,
};
use noirc_frontend::{debug::DebugInstrumenter, graph::CrateName};

use crate::errors::CliError;

use super::NargoConfig;

/// Perform formal verification on a program
#[derive(Debug, Clone, Args)]
#[clap(visible_alias = "fv")]
pub(crate) struct FormalVerifyCommand {
    /// The name of the package to formally verify
    #[clap(long, conflicts_with = "workspace")]
    package: Option<CrateName>,

    /// Formally verify all packages in the workspace
    #[clap(long, conflicts_with = "package")]
    workspace: bool,

    // This is necessary for compile functions
    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run(args: FormalVerifyCommand, config: NargoConfig) -> Result<(), CliError> {
    let toml_path = get_package_manifest(&config.program_dir)?;
    let default_selection =
        if args.workspace { PackageSelection::All } else { PackageSelection::DefaultOrAll };
    let selection = args.package.map_or(default_selection, PackageSelection::Selected);
    let workspace = resolve_workspace_from_toml(
        &toml_path,
        selection,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
    )?;

    let mut workspace_file_manager = file_manager_with_stdlib(&workspace.root_dir);
    insert_all_files_for_workspace_into_file_manager(&workspace, &mut workspace_file_manager);
    let parsed_files = parse_all(&workspace_file_manager);

    let binary_packages = workspace.into_iter().filter(|package| package.is_binary());
    for package in binary_packages {
        let (mut context, crate_id) =
            prepare_package(&workspace_file_manager, &parsed_files, package);
        link_to_debug_crate(&mut context, crate_id);
        context.debug_instrumenter = DebugInstrumenter::default();
        context.package_build_path = workspace.package_build_path(package);
        // Note: This is the only important line in this function. Everything else is equivalent to the compile command.
        // (Except saving the result in a file.)
        context.perform_formal_verification = true;

        let formal_verification_result = noirc_driver::compile_main(
            &mut context,
            crate_id,
            &args.compile_options,
            None,
            false,
        );

        report_errors(
            formal_verification_result,
            &workspace_file_manager,
            args.compile_options.deny_warnings,
            true, // We don't want to report compile related warnings
        )?;
    }

    Ok(())
}
