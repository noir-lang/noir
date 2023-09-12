use acvm_backend_barretenberg::backends_directory;
use clap::Args;

use crate::errors::CliError;

/// Prints the list of currently installed backends
#[derive(Debug, Clone, Args)]
pub(crate) struct LsCommand;

pub(crate) fn run(_args: LsCommand) -> Result<(), CliError> {
    for backend in get_available_backends() {
        println!("{backend}");
    }

    Ok(())
}

pub(super) fn get_available_backends() -> Vec<String> {
    let backend_directory_contents = std::fs::read_dir(backends_directory())
        .expect("Could not read backends directory contents");

    // TODO: Highlight the currently active backend.
    backend_directory_contents
        .into_iter()
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.is_dir() {
                path.file_name().map(|name| name.to_string_lossy().to_string())
            } else {
                None
            }
        })
        .collect()
}
