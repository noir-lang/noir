#![forbid(unsafe_code)]
#![warn(unused_extern_crates)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]

//! Nargo is the package manager for Noir
//! This name was used because it sounds like `cargo` and
//! Noir Package Manager abbreviated is npm, which is already taken.

use fm::FileManager;
use nargo::package::{Dependency, Package};
use noirc_driver::{add_dep, create_local_crate, create_non_local_crate};
use noirc_frontend::{
    graph::{CrateGraph, CrateId, CrateName, CrateType},
    hir::Context,
};
use std::{
    collections::BTreeMap,
    fs::ReadDir,
    path::{Path, PathBuf},
};

use errors::ManifestError;

mod backends;
pub mod cli;
mod errors;
mod git;
mod manifest;

fn nargo_crates() -> PathBuf {
    dirs::home_dir().unwrap().join("nargo")
}

/// Returns the path of the root directory of the package containing `current_path`.
///
/// Returns a `CliError` if no parent directories of `current_path` contain a manifest file.
fn find_package_root(current_path: &Path) -> Result<PathBuf, ManifestError> {
    let manifest_path = find_package_manifest(current_path)?;

    let package_root =
        manifest_path.parent().expect("infallible: manifest file path can't be root directory");

    Ok(package_root.to_path_buf())
}

/// Returns the path of the manifest file (`Nargo.toml`) of the package containing `current_path`.
///
/// Returns a `CliError` if no parent directories of `current_path` contain a manifest file.
fn find_package_manifest(current_path: &Path) -> Result<PathBuf, ManifestError> {
    current_path
        .ancestors()
        .find_map(|dir| find_file(dir, "Nargo", "toml"))
        .ok_or_else(|| ManifestError::MissingFile(current_path.to_path_buf()))
}

fn lib_or_bin(root_dir: impl AsRef<Path>) -> Result<(PathBuf, CrateType), ManifestError> {
    let current_path = root_dir.as_ref();
    // A library has a lib.nr and a binary has a main.nr
    // You cannot have both.
    let src_path = find_dir(current_path, "src")
        .ok_or_else(|| ManifestError::NoSourceDir(current_path.to_path_buf()))?;

    let lib_nr_path = find_file(&src_path, "lib", "nr");
    let bin_nr_path = find_file(&src_path, "main", "nr");
    match (lib_nr_path, bin_nr_path) {
        (Some(_), Some(_)) => Err(ManifestError::ContainsMultipleCrates),
        (None, Some(path)) => Ok((path, CrateType::Binary)),
        (Some(path), None) => Ok((path, CrateType::Library)),
        (None, None) => Err(ManifestError::ContainsZeroCrates),
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

fn prepare_dependencies(
    context: &mut Context,
    parent_crate: CrateId,
    dependencies: BTreeMap<CrateName, Dependency>,
) {
    for (dep_name, dep) in dependencies.into_iter() {
        match dep {
            Dependency::Remote { entry_path, crate_type, workspace }
            | Dependency::Local { entry_path, crate_type, workspace } => {
                let crate_id = create_non_local_crate(context, &entry_path, crate_type);
                add_dep(context, parent_crate, crate_id, dep_name);
                for package in &workspace {
                    prepare_dependencies(context, crate_id, package.dependencies.to_owned());
                }
            }
        }
    }
}

fn prepare_package(package: &Package) -> (Context, CrateId) {
    let fm = FileManager::new(&package.root_dir);
    let graph = CrateGraph::default();
    let mut context = Context::new(fm, graph);

    let crate_id = create_local_crate(&mut context, &package.entry_path, package.crate_type);

    prepare_dependencies(&mut context, crate_id, package.dependencies.to_owned());

    (context, crate_id)
}
