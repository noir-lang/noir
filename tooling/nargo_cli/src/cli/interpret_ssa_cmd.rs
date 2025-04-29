use crate::cli::LockType;
use nargo::{prepare_package, workspace::Workspace};
use nargo_toml::PackageSelection;
use noirc_driver::CompileOptions;

use super::{
    CliError, PackageOptions, WorkspaceCommand,
    compile_cmd::{compile_workspace_full, parse_workspace},
};

/// Interpret the resulting SSA of a program
#[derive(Debug, Clone, clap::Args)]
pub(crate) struct InterpretSsaCommand {
    #[clap(flatten)]
    pub(super) package_options: PackageOptions,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

impl WorkspaceCommand for InterpretSsaCommand {
    fn package_selection(&self) -> PackageSelection {
        self.package_options.package_selection()
    }

    fn lock_type(&self) -> LockType {
        // Compiles artifacts.
        LockType::Exclusive
    }
}

pub(crate) fn run(args: InterpretSsaCommand, workspace: Workspace) -> Result<(), CliError> {
    // Compile the full workspace in order to generate any build artifacts.
    compile_workspace_full(&workspace, &args.compile_options)?;

    let binary_packages = workspace.into_iter().filter(|package| package.is_binary());
    for package in binary_packages {
        let _program_artifact_path = workspace.package_build_path(package);

        let (file_manager, parsed_files) = parse_workspace(&workspace);
        let (mut context, crate_id) = prepare_package(&file_manager, &parsed_files, package);

        let main = context.get_main_function(&crate_id).ok_or_else(|| {
            // TODO(#2155): This error might be a better to exist in Nargo
            CliError::Generic(
                "cannot compile crate into a program as it does not contain a `main` function"
                    .to_string(),
            )
        })?;

        match noirc_driver::compile_to_ssa(&mut context, &args.compile_options, main) {
            Ok((_ssa, _brillig, warnings)) => {
                for warning in warnings {
                    eprintln!("{warning:?}");
                }
            }
            Err(error) => panic!("{error:?}"),
        }
    }
    Ok(())
}
