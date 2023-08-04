use crate::errors::CliError;

use super::{init_cmd::initialize_project, NargoConfig};
use acvm::Backend;
use clap::Args;
use std::path::PathBuf;

/// Create a Noir project in a new directory.
#[derive(Debug, Clone, Args)]
pub(crate) struct NewCommand {
    /// The path to save the new project
    path: PathBuf,

    /// Name of the package [default = package directory name]
    #[clap(long)]
    name: Option<String>,
}

pub(crate) fn run<B: Backend>(
    // Backend is currently unused, but we might want to use it to inform the "new" template in the future
    _backend: &B,
    args: NewCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {
    let package_dir = config.program_dir.join(&args.path);

    if package_dir.exists() {
        return Err(CliError::DestinationAlreadyExists(package_dir));
    }

    let package_name =
        args.name.unwrap_or_else(|| args.path.file_name().unwrap().to_str().unwrap().to_owned());
    initialize_project(package_dir, &package_name);
    Ok(())
}
