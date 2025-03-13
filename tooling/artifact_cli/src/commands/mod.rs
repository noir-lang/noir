//! This module is for commands that we might want to invoke from `nargo` as-is.
use std::path::PathBuf;

use color_eyre::eyre;
use eyre::eyre;

pub mod execute_cmd;

/// Parses a path and turns it into an absolute one by joining to the current directory,
/// then normalizes it.
fn parse_and_normalize_path(path: &str) -> eyre::Result<PathBuf> {
    use fm::NormalizePath;
    let mut path: PathBuf = path.parse().map_err(|e| eyre!("failed to parse path: {e}"))?;
    if !path.is_absolute() {
        path = std::env::current_dir().unwrap().join(path).normalize();
    }
    Ok(path)
}
