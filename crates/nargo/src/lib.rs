use noirc_frontend::graph::CrateType;
use std::path::{Path, PathBuf};

use crate::errors::CliError;
// Nargo is the package manager for Noir
// This name was used because it sounds like `cargo` and
// Noir Package Manager abbreviated is npm, which is already taken.

fn nargo_crates() -> PathBuf {
    dirs::home_dir().unwrap().join("nargo")
}

mod backends;
pub mod cli;
mod constants;
mod errors;
mod git;
mod resolver;
mod toml;

/// Searches for the Nargo.toml file
///
/// XXX: In the end, this should find the root of the project and check
/// for the Nargo.toml file there
/// However, it should only do this after checking the current path
/// This allows the use of workspace settings in the future.
fn find_package_config(current_path: &Path) -> Result<PathBuf, CliError> {
    match fm::find_file(current_path, "Nargo", "toml") {
        Some(p) => Ok(p),
        None => Err(CliError::Generic(format!(
            "cannot find a Nargo.toml in {}",
            current_path.display()
        ))),
    }
}

fn lib_or_bin(current_path: &Path) -> Result<(PathBuf, CrateType), CliError> {
    // A library has a lib.nr and a binary has a main.nr
    // You cannot have both.
    let src_path = match fm::find_dir(current_path, "src") {
        Some(path) => path,
        None => {
            return Err(CliError::Generic(format!(
                "cannot find src file in path {}",
                current_path.display()
            )))
        }
    };
    let lib_nr_path = fm::find_file(&src_path, "lib", "nr");
    let bin_nr_path = fm::find_file(&src_path, "main", "nr");
    match (lib_nr_path, bin_nr_path) {
        (Some(_), Some(_)) => Err(CliError::Generic(
            "package cannot contain both a `lib.nr` and a `main.nr`".to_owned(),
        )),
        (None, Some(path)) => Ok((path, CrateType::Binary)),
        (Some(path), None) => Ok((path, CrateType::Library)),
        (None, None) => Err(CliError::Generic(
            "package must contain either a `lib.nr`(Library) or a `main.nr`(Binary).".to_owned(),
        )),
    }
}
