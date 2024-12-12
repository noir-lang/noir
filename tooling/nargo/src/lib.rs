#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]

//! Nargo is the package manager for Noir
//! This name was used because it sounds like `cargo` and
//! Noir Package Manager abbreviated is npm, which is already taken.

pub mod constants;
pub mod errors;
pub mod foreign_calls;
pub mod ops;
pub mod package;
pub mod workspace;

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    path::PathBuf,
};

use fm::{FileManager, FILE_EXTENSION};
use noirc_driver::{add_dep, prepare_crate, prepare_dependency};
use noirc_frontend::{
    graph::{CrateId, CrateName},
    hir::{def_map::parse_file, Context, ParsedFiles},
};
use package::{Dependency, Package};
use rayon::prelude::*;
use walkdir::WalkDir;

pub use self::errors::NargoError;
pub use self::foreign_calls::print::PrintOutput;

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
    insert_all_files_for_workspace_into_file_manager_with_overrides(
        workspace,
        file_manager,
        &HashMap::new(),
    );
}

pub fn insert_all_files_for_workspace_into_file_manager_with_overrides(
    workspace: &workspace::Workspace,
    file_manager: &mut FileManager,
    overrides: &HashMap<&std::path::Path, &str>,
) {
    let mut processed_entry_paths = HashSet::new();
    for package in workspace.clone().into_iter() {
        insert_all_files_for_package_into_file_manager(
            package,
            file_manager,
            overrides,
            &mut processed_entry_paths,
        );
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
    overrides: &HashMap<&std::path::Path, &str>,
    processed_entry_paths: &mut HashSet<PathBuf>,
) {
    if processed_entry_paths.contains(&package.entry_path) {
        return;
    }
    processed_entry_paths.insert(package.entry_path.clone());

    // Start off at the entry path and read all files in the parent directory.
    let entry_path_parent = package
        .entry_path
        .parent()
        .unwrap_or_else(|| panic!("The entry path is expected to be a single file within a directory and so should have a parent {:?}", package.entry_path));

    for entry in WalkDir::new(entry_path_parent) {
        let Ok(entry) = entry else {
            continue;
        };

        if !entry.file_type().is_file() {
            continue;
        }

        if !entry.path().extension().map_or(false, |ext| ext == FILE_EXTENSION) {
            continue;
        };

        let path = entry.into_path();

        // Avoid reading the source if the file is already there
        if file_manager.has_file(&path) {
            continue;
        }

        let source = if let Some(src) = overrides.get(path.as_path()) {
            src.to_string()
        } else {
            std::fs::read_to_string(path.as_path())
                .unwrap_or_else(|_| panic!("could not read file {:?} into string", path))
        };

        file_manager.add_file_with_source(path.as_path(), source);
    }

    insert_all_files_for_packages_dependencies_into_file_manager(
        package,
        file_manager,
        overrides,
        processed_entry_paths,
    );
}

// Inserts all files for the dependencies of the package into the file manager
// too
fn insert_all_files_for_packages_dependencies_into_file_manager(
    package: &Package,
    file_manager: &mut FileManager,
    overrides: &HashMap<&std::path::Path, &str>,
    processed_entry_paths: &mut HashSet<PathBuf>,
) {
    for (_, dep) in package.dependencies.iter() {
        match dep {
            Dependency::Local { package } | Dependency::Remote { package } => {
                insert_all_files_for_package_into_file_manager(
                    package,
                    file_manager,
                    overrides,
                    processed_entry_paths,
                );
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
