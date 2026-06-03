use crate::errors::CliError;

use clap::Args;
use nargo::workspace::Workspace;
use nargo_toml::PackageSelection;

use super::{LockType, PackageOptions, WorkspaceCommand};

/// Download and install the dependencies of a package
#[derive(Debug, Clone, Args)]
pub(crate) struct InstallCommand {
    #[clap(flatten)]
    pub(super) package_options: PackageOptions,
}

impl WorkspaceCommand for InstallCommand {
    fn package_selection(&self) -> PackageSelection {
        self.package_options.package_selection()
    }

    fn lock_type(&self) -> LockType {
        // The `LockType` lock guards a workspace member's `target/` artifacts, which this command
        // never writes. Git dependencies are downloaded into the global cache during workspace
        // resolution, which serializes concurrent runs with its own `.package-cache` lock before
        // any `LockType` lock would be taken, so `None` is correct here.
        LockType::None
    }
}

pub(crate) fn run(_args: InstallCommand, _workspace: Workspace) -> Result<(), CliError> {
    // Resolving the workspace (which happens before this runs) downloads any missing git
    // dependencies into the global cache, with `git` printing its own progress as it clones.
    // There is nothing left to do here.
    Ok(())
}
