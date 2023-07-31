use crate::errors::CliError;

use super::{
    init_cmd::{initialize_project, InitCommand},
    NargoConfig,
};
use acvm::Backend;
use clap::Args;
use nargo::package::PackageType;
use std::path::PathBuf;

/// Create a Noir project in a new directory.
#[derive(Debug, Clone, Args)]
pub(crate) struct NewCommand {
    /// Name of the package
    package_name: String,
    /// The path to save the new project
    path: Option<PathBuf>,

    #[clap(flatten)]
    init_config: InitCommand,
}

pub(crate) fn run<B: Backend>(
    // Backend is currently unused, but we might want to use it to inform the "new" template in the future
    _backend: &B,
    args: NewCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {
    let package_dir = config.program_dir.join(args.package_name);

    if package_dir.exists() {
        return Err(CliError::DestinationAlreadyExists(package_dir));
    }

    let package_type =
        if args.init_config.lib { PackageType::Library } else { PackageType::Binary };
    initialize_project(package_dir, package_type);
    Ok(())
}
