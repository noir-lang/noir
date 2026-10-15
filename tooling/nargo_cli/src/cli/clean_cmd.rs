use std::fs;

use clap::Args;

use crate::errors::CliError;

use super::NargoConfig;

/// Remove the target directory in the given program directory to clear build artifacts.
#[derive(Debug, Clone, Args)]
pub(crate) struct CleanCommand {}

pub(crate) fn run(_args: CleanCommand, config: NargoConfig) -> Result<(), CliError> {
    let target_dir = match config.target_dir {
        Some(target_dir) => target_dir,
        None => config.program_dir.join("target"),
    };

    // Exit early if the target directory doesn't exist so that `nargo clean`
    // doesn't fail on a project which hasn't been built yet.
    if !target_dir.exists() {
        return Ok(());
    }

    fs::remove_dir_all(&target_dir).map_err(|err| CliError::Generic(err.to_string()))?;

    Ok(())
}
