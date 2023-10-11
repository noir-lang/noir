use std::path::PathBuf;

use clap::Args;

use backend_interface::{backends_directory, download_backend, run_backend_installation_script};

use crate::errors::{BackendError, CliError};

use super::ls_cmd::get_available_backends;

/// Install a new backend from a URL.
#[derive(Debug, Clone, Args)]
pub(crate) struct InstallCommand {
    /// The name of the backend to install.
    backend: String,

    /// The URL from which to download the backend.
    url: String,

    #[arg(long)]
    bash: bool,
}

pub(crate) fn run(args: InstallCommand) -> Result<(), CliError> {
    let installed_backends = get_available_backends();

    if installed_backends.contains(&args.backend) {
        return Err(BackendError::AlreadyInstalled(args.backend).into());
    }

    if args.bash {
        run_backend_installation_script(
            &PathBuf::from(args.url),
            &backends_directory().join(args.backend).join("backend_binary"),
        )
        .map_err(BackendError::from)?;
    } else {
        download_backend(
            &args.url,
            &backends_directory().join(args.backend).join("backend_binary"),
        )
        .map_err(BackendError::from)?;
    }
    Ok(())
}
