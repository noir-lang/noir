use iter_extended::vecmap;
use nargo::artifacts::program::PreprocessedProgram;
use nargo::package::Package;
use nargo::prepare_package;
use nargo::workspace::Workspace;
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_driver::compile_no_check;
use noirc_driver::CompileOptions;
use noirc_driver::CompiledProgram;
use noirc_driver::NOIR_ARTIFACT_VERSION_STRING;
use noirc_frontend::graph::CrateName;

use clap::Args;

use crate::backends::Backend;
use crate::errors::CliError;

use super::check_cmd::check_crate_and_report_errors;

use super::fs::program::save_program_to_file;
use super::NargoConfig;

/// Compile the program and its secret execution trace into ACIR format
#[derive(Debug, Clone, Args)]
pub(crate) struct ExportCommand {
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
    _backend: &Backend,
    args: ExportCommand,
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

    let library_packages: Vec<_> =
        workspace.into_iter().filter(|package| package.is_library()).collect();

    compile_program(&workspace, library_packages[0], &args.compile_options)?;

    Ok(())
}

fn compile_program(
    workspace: &Workspace,
    package: &Package,
    compile_options: &CompileOptions,
) -> Result<(), CliError> {
    let (mut context, crate_id) = prepare_package(package);
    check_crate_and_report_errors(
        &mut context,
        crate_id,
        compile_options.deny_warnings,
        compile_options.disable_macros,
        compile_options.silence_warnings,
    )?;

    let exported_functions = context.get_all_exported_functions_in_crate(&crate_id);

    let exported_programs =
        vecmap(exported_functions, |(function_name, function_id)| -> (String, CompiledProgram) {
            let program = compile_no_check(&context, compile_options, function_id, None, false)
                .expect("heyooo");

            (function_name, program)
        });

    let export_dir = workspace.target_directory_path().parent().unwrap().join("export");
    for (function_name, program) in exported_programs {
        let preprocessed_program = PreprocessedProgram {
            hash: program.hash,
            abi: program.abi,
            noir_version: program.noir_version,
            bytecode: program.circuit,
        };

        save_program_to_file(&preprocessed_program, &function_name.parse().unwrap(), &export_dir);
    }
    Ok(())
}
