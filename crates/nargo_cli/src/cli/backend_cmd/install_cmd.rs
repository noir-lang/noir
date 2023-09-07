use clap::Args;

use acvm_backend_barretenberg::{backends_directory, download_backend};

use crate::errors::CliError;

use super::ls_cmd::get_available_backends;

/// Install a new backend
#[derive(Debug, Clone, Args)]
pub(crate) struct InstallCommand {
    backend: String,

    url: String,
}

pub(crate) fn run(args: InstallCommand) -> Result<(), CliError> {
    let installed_backends = get_available_backends();

    assert!(!installed_backends.contains(&args.backend), "backend is already installed");

    download_backend(&args.url, &backends_directory().join(args.backend).join("backend_binary"));

    Ok(())
}
