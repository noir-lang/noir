use std::{
    env,
    fs::remove_dir_all,
    path::{Path, PathBuf},
};

use acvm::Backend;
use clap::Args;
use nargo::constants::TARGET_DIR;

use crate::errors::CliError;

use super::NargoConfig;

/// Remove both target directory and nargo cache
#[derive(Debug, Clone, Args)]
pub(crate) struct CleanCommand {
    /// remove cache (including the crs) only
    #[arg(short, long)]
    cache: bool,

    /// remove target directory only
    #[arg(short, long)]
    target_dir: bool,
}

pub(crate) fn run<B: Backend>(
    // Backend is not used in clean command.
    _backend: &B,
    args: CleanCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {
    let target_dir = config.program_dir.join(TARGET_DIR);
    let cache_dir = match env::var("NARGO_BACKEND_CACHE_DIR") {
        Ok(cache_dir) => PathBuf::from(cache_dir),
        Err(_) => dirs::home_dir().unwrap().join(".nargo/backends"),
    };

    if args.cache {
        return Ok(clean_entire_folder(cache_dir)?);
    }

    if args.target_dir {
        return Ok(clean_entire_folder(target_dir)?);
    }

    clean_entire_folder(cache_dir)?;
    clean_entire_folder(target_dir)?;

    Ok(())
}

pub(crate) fn clean_entire_folder<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    if std::fs::symlink_metadata(&path).is_err() {
        return Ok(());
    }

    remove_dir_all(path)
}
