use nargo::workspace::Workspace;
use nargo_toml::PackageSelection;
use noirc_driver::CompileOptions;

use super::{CliError, PackageOptions, WorkspaceCommand};

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
    todo!()
}
