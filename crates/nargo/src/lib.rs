#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]

// Necessary as we use `color_eyre` in `main.rs`.
use color_eyre as _;

use noirc_frontend::graph::CrateType;
use std::{
    fs::ReadDir,
    path::{Path, PathBuf},
};

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
mod manifest;
mod resolver;

/// Returns the path of the root directory of the package containing `current_path`.
///
/// Returns a `CliError` if no parent directories of `current_path` contain a manifest file.
fn find_package_root(current_path: &Path) -> Result<PathBuf, CliError> {
    let manifest_path = find_package_manifest(current_path)?;

    let package_root =
        manifest_path.parent().expect("infallible: manifest file path can't be root directory");

    Ok(package_root.to_path_buf())
}

/// Returns the path of the manifest file (`Nargo.toml`) of the package containing `current_path`.
///
/// Returns a `CliError` if no parent directories of `current_path` contain a manifest file.
fn find_package_manifest(current_path: &Path) -> Result<PathBuf, CliError> {
    current_path.ancestors().find_map(|dir| find_file(dir, "Nargo", "toml")).ok_or_else(|| {
        CliError::Generic(format!(
            "could not find Nargo.toml in {} or any parent directory",
            current_path.display()
        ))
    })
}

fn lib_or_bin(current_path: &Path) -> Result<(PathBuf, CrateType), CliError> {
    // A library has a lib.nr and a binary has a main.nr
    // You cannot have both.
    let src_path = match find_dir(current_path, "src") {
        Some(path) => path,
        None => {
            return Err(CliError::Generic(format!(
                "cannot find src file in path {}",
                current_path.display()
            )))
        }
    };
    let lib_nr_path = find_file(&src_path, "lib", "nr");
    let bin_nr_path = find_file(&src_path, "main", "nr");
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

// Looks for file named `file_name` in path
fn find_file<P: AsRef<Path>>(path: P, file_name: &str, extension: &str) -> Option<PathBuf> {
    let entries = list_files_and_folders_in(path)?;
    let file_name = format!("{file_name}.{extension}");

    find_artifact(entries, &file_name)
}

// Looks for directory named `dir_name` in path
fn find_dir<P: AsRef<Path>>(path: P, dir_name: &str) -> Option<PathBuf> {
    let entries = list_files_and_folders_in(path)?;
    find_artifact(entries, dir_name)
}

// There is no distinction between files and folders
fn find_artifact(entries: ReadDir, artifact_name: &str) -> Option<PathBuf> {
    let entry = entries
        .into_iter()
        .flatten()
        .find(|entry| entry.file_name().to_str() == Some(artifact_name))?;

    Some(entry.path())
}

fn list_files_and_folders_in<P: AsRef<Path>>(path: P) -> Option<ReadDir> {
    std::fs::read_dir(path).ok()
}
