use crate::errors::CliError;

use super::{init_cmd::initialize_project, NargoConfig};
use clap::Args;
use nargo::package::{CrateName, PackageType};
use std::path::PathBuf;

/// Create a Noir project in a new directory.
#[derive(Debug, Clone, Args)]
pub(crate) struct NewCommand {
    /// The path to save the new project
    path: PathBuf,

    /// Name of the package [default: package directory name]
    #[clap(long)]
    name: Option<CrateName>,

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

pub(crate) fn run(args: NewCommand, config: NargoConfig) -> Result<(), CliError> {
    let package_dir = config.program_dir.join(&args.path);

    if package_dir.exists() {
        return Err(CliError::DestinationAlreadyExists(package_dir));
    }

    let package_name = match args.name {
        Some(name) => name,
        None => {
            let name = args.path.file_name().unwrap().to_str().unwrap();
            name.parse().map_err(|_| CliError::InvalidPackageName(name.into()))?
        }
    };
    let package_type = if args.lib {
        PackageType::Library
    } else if args.contract {
        PackageType::Contract
    } else {
        PackageType::Binary
    };
    initialize_project(package_dir, package_name, package_type);
    Ok(())
}
