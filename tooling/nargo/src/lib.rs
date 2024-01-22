#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]

//! Nargo is the package manager for Noir
//! This name was used because it sounds like `cargo` and
//! Noir Package Manager abbreviated is npm, which is already taken.

pub mod artifacts;
pub mod constants;
pub mod errors;
pub mod ops;
pub mod package;
pub mod workspace;

use std::collections::BTreeMap;

use fm::FileManager;
use noirc_driver::{add_dep, prepare_crate, prepare_dependency};
use noirc_frontend::{
    graph::{CrateId, CrateName},
    hir::{def_map::parse_file, Context, ParsedFiles},
};
use package::{Dependency, Package};
use rayon::prelude::*;

pub use self::errors::NargoError;

pub fn prepare_dependencies(
    context: &mut Context,
    parent_crate: CrateId,
    dependencies: &BTreeMap<CrateName, Dependency>,
) {
    for (dep_name, dep) in dependencies.iter() {
        match dep {
            Dependency::Remote { package } | Dependency::Local { package } => {
                let crate_id = prepare_dependency(context, &package.entry_path);
                add_dep(context, parent_crate, crate_id, dep_name.clone());
                prepare_dependencies(context, crate_id, &package.dependencies);
            }
        }
    }
}

pub fn insert_all_files_for_workspace_into_file_manager(
    workspace: &workspace::Workspace,
    file_manager: &mut FileManager,
) {
    for package in workspace.clone().into_iter() {
        insert_all_files_for_package_into_file_manager(package, file_manager);
    }
}
// We will pre-populate the file manager with all the files in the package
// This is so that we can avoid having to read from disk when we are compiling
//
// This does not require parsing because we are interested in the files under the src directory
// it may turn out that we do not need to include some Noir files that we add to the file
// manager
fn insert_all_files_for_package_into_file_manager(
    package: &Package,
    file_manager: &mut FileManager,
) {
    // Start off at the entry path and read all files in the parent directory.
    let entry_path_parent = package
        .entry_path
        .parent()
        .unwrap_or_else(|| panic!("The entry path is expected to be a single file within a directory and so should have a parent {:?}", package.entry_path))
        .clone();

    // Get all files in the package and add them to the file manager
    let paths =
        get_all_paths_in_dir(entry_path_parent).expect("could not get all paths in the package");
    for path in paths {
        let source = std::fs::read_to_string(path.as_path())
            .unwrap_or_else(|_| panic!("could not read file {:?} into string", path));
        file_manager.add_file_with_source(path.as_path(), source);
    }

    insert_all_files_for_packages_dependencies_into_file_manager(package, file_manager);
}

// Inserts all files for the dependencies of the package into the file manager
// too
fn insert_all_files_for_packages_dependencies_into_file_manager(
    package: &Package,
    file_manager: &mut FileManager,
) {
    for (_, dep) in package.dependencies.iter() {
        match dep {
            Dependency::Local { package } | Dependency::Remote { package } => {
                insert_all_files_for_package_into_file_manager(package, file_manager);
                insert_all_files_for_packages_dependencies_into_file_manager(package, file_manager);
            }
        }
    }
}

pub fn parse_all(file_manager: &FileManager) -> ParsedFiles {
    file_manager
        .as_file_map()
        .all_file_ids()
        .par_bridge()
        .filter(|&&file_id| {
            let file_path = file_manager.path(file_id).expect("expected file to exist");
            let file_extension =
                file_path.extension().expect("expected all file paths to have an extension");
            file_extension == "nr"
        })
        .map(|&file_id| (file_id, parse_file(file_manager, file_id)))
        .collect()
}

pub fn prepare_package<'file_manager, 'parsed_files>(
    file_manager: &'file_manager FileManager,
    parsed_files: &'parsed_files ParsedFiles,
    package: &Package,
) -> (Context<'file_manager, 'parsed_files>, CrateId) {
    let mut context = Context::from_ref_file_manager(file_manager, parsed_files);

    let crate_id = prepare_crate(&mut context, &package.entry_path);

    prepare_dependencies(&mut context, crate_id, &package.dependencies);

    (context, crate_id)
}

// Get all paths in the directory and subdirectories.
//
// Panics: If the path is not a path to a directory.
//
// TODO: Along with prepare_package, this function is an abstraction leak
// TODO: given that this crate should not know about the file manager.
// TODO: We can clean this up in a future refactor
fn get_all_paths_in_dir(dir: &std::path::Path) -> std::io::Result<Vec<std::path::PathBuf>> {
    assert!(dir.is_dir(), "directory {dir:?} is not a path to a directory");

    let mut paths = Vec::new();

    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let mut sub_paths = get_all_paths_in_dir(&path)?;
                paths.append(&mut sub_paths);
            } else {
                paths.push(path);
            }
        }
    }

    Ok(paths)
}

#[cfg(test)]
mod tests {
    use crate::get_all_paths_in_dir;
    use std::{
        fs::{self, File},
        path::Path,
    };
    use tempfile::tempdir;

    fn create_test_dir_structure(temp_dir: &Path) -> std::io::Result<()> {
        fs::create_dir(temp_dir.join("sub_dir1"))?;
        File::create(temp_dir.join("sub_dir1/file1.txt"))?;
        fs::create_dir(temp_dir.join("sub_dir2"))?;
        File::create(temp_dir.join("sub_dir2/file2.txt"))?;
        File::create(temp_dir.join("file3.txt"))?;
        Ok(())
    }

    #[test]
    fn test_get_all_paths_in_dir() {
        let temp_dir = tempdir().expect("could not create a temporary directory");
        create_test_dir_structure(temp_dir.path())
            .expect("could not create test directory structure");

        let paths = get_all_paths_in_dir(temp_dir.path())
            .expect("could not get all paths in the test directory");

        // This should be the paths to all of the files in the directory and the subdirectory
        let expected_paths = vec![
            temp_dir.path().join("file3.txt"),
            temp_dir.path().join("sub_dir1/file1.txt"),
            temp_dir.path().join("sub_dir2/file2.txt"),
        ];

        assert_eq!(paths.len(), expected_paths.len());
        for path in expected_paths {
            assert!(paths.contains(&path));
        }
    }
}
