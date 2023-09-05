use acvm_backend_barretenberg::backends_directory;
use clap::Args;

use crate::errors::CliError;

/// Deletes all backends
#[derive(Debug, Clone, Args)]
pub(crate) struct RmAllCommand;

pub(crate) fn run(_args: RmAllCommand) -> Result<(), CliError> {
    remove_all_backends().map_err(|io_error| CliError::Generic(io_error.to_string()))
}

pub(super) fn remove_all_backends() -> std::io::Result<()> {
    let backend_directory_contents = std::fs::read_dir(backends_directory()).unwrap();
    for entry in backend_directory_contents {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_dir() {
                std::fs::remove_dir_all(path)?;
            }
        }
    }
    Ok(())
}
