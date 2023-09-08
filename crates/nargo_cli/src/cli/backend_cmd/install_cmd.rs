use clap::Args;

use acvm_backend_barretenberg::{backends_directory, download_backend};

use crate::errors::{BackendError, CliError};

use super::ls_cmd::get_available_backends;

/// Install a new backend from a URL.
#[derive(Debug, Clone, Args)]
pub(crate) struct InstallCommand {
    /// The name of the backend to install.
    backend: String,

    /// The URL from which to download the backend.
    url: String,
}

pub(crate) fn run(args: InstallCommand) -> Result<(), CliError> {
    let installed_backends = get_available_backends();

    if installed_backends.contains(&args.backend) {
        return Err(BackendError::AlreadyInstalled(args.backend).into());
    }

    download_backend(&args.url, &backends_directory().join(args.backend).join("backend_binary"))
        .map_err(BackendError::from)?;

    Ok(())
}
