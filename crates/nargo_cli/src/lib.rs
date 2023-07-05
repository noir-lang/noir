#![forbid(unsafe_code)]
#![warn(unused_extern_crates)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]

//! Nargo is the package manager for Noir
//! This name was used because it sounds like `cargo` and
//! Noir Package Manager abbreviated is npm, which is already taken.

use std::{
    fs::ReadDir,
    path::{Path, PathBuf},
};

mod backends;
pub mod cli;
mod constants;
mod errors;
mod git;
mod manifest;
mod resolver;

use nargo::manifest::InvalidPackageError;

fn nargo_crates() -> PathBuf {
    dirs::home_dir().unwrap().join("nargo")
}

/// Returns the path of the root directory of the package containing `current_path`.
///
/// Returns a `CliError` if no parent directories of `current_path` contain a manifest file.
fn find_package_root(current_path: &Path) -> Result<PathBuf, InvalidPackageError> {
    let manifest_path = find_package_manifest(current_path)?;

    let package_root =
        manifest_path.parent().expect("infallible: manifest file path can't be root directory");

    Ok(package_root.to_path_buf())
}

/// Returns the path of the manifest file (`Nargo.toml`) of the package containing `current_path`.
///
/// Returns a `CliError` if no parent directories of `current_path` contain a manifest file.
fn find_package_manifest(current_path: &Path) -> Result<PathBuf, InvalidPackageError> {
    current_path
        .ancestors()
        .find_map(|dir| find_file(dir, "Nargo", "toml"))
        .ok_or_else(|| InvalidPackageError::MissingManifestFile(current_path.to_path_buf()))
}

fn lib_or_bin(current_path: impl AsRef<Path>) -> Result<PathBuf, InvalidPackageError> {
    let current_path = current_path.as_ref();
    // A library has a lib.nr and a binary has a main.nr
    // You cannot have both.
    let src_path = find_dir(current_path, "src")
        .ok_or_else(|| InvalidPackageError::NoSourceDir(current_path.to_path_buf()))?;

    let lib_nr_path = find_file(&src_path, "lib", "nr");
    let bin_nr_path = find_file(&src_path, "main", "nr");
    match (lib_nr_path, bin_nr_path) {
        (Some(_), Some(_)) => Err(InvalidPackageError::ContainsMultipleCrates),
        (None, Some(path)) => Ok(path),
        (Some(path), None) => Ok(path),
        (None, None) => Err(InvalidPackageError::ContainsZeroCrates),
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
