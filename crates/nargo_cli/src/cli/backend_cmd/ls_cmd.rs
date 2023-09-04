use acvm_backend_barretenberg::backends_directory;
use clap::Args;

use crate::errors::CliError;

/// Checks the constraint system for errors
#[derive(Debug, Clone, Args)]
pub(crate) struct LsCommand;

pub(crate) fn run(_args: LsCommand) -> Result<(), CliError> {
    let backend_directory_contents = std::fs::read_dir(backends_directory()).unwrap();

    let backend_directories: Vec<std::path::PathBuf> = backend_directory_contents
        .into_iter()
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.is_dir() {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    for backend in backend_directories {
        println!("{}", backend.file_name().unwrap().to_str().unwrap());
    }

    Ok(())
}
