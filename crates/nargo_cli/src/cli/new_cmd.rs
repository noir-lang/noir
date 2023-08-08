use crate::errors::CliError;

use super::{init_cmd::initialize_project, NargoConfig};
use acvm::Backend;
use clap::Args;
use nargo::package::PackageType;
use std::path::PathBuf;

/// Create a Noir project in a new directory.
#[derive(Debug, Clone, Args)]
pub(crate) struct NewCommand {
    /// The path to save the new project
    path: PathBuf,

    /// Name of the package [default: package directory name]
    #[clap(long)]
    name: Option<String>,

    /// Use a library template
    #[arg(long, conflicts_with = "bin", conflicts_with = "contract")]
    pub(crate) lib: bool,

    /// Use a binary template [default]
    #[arg(long, conflicts_with = "lib", conflicts_with = "contract")]
    pub(crate) bin: bool,

    /// Use a contract template
    #[arg(long, conflicts_with = "lib", conflicts_with = "bin")]
    pub(crate) contract: bool,
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
    let package_type = if args.lib {
        PackageType::Library
    } else if args.contract {
        PackageType::Contract
    } else {
        PackageType::Binary
    };
    initialize_project(package_dir, &package_name, package_type);
    Ok(())
}
