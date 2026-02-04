#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]

//! Nargo is the package manager for Noir
//! This name was used because it sounds like `cargo` and
//! Noir Package Manager abbreviated is npm, which is already taken.

pub mod constants;
pub mod errors;
pub mod foreign_calls;
pub mod ops;
pub mod package;
pub mod workspace;

pub use self::errors::NargoError;
pub use self::ops::FuzzExecutionConfig;
pub use self::ops::FuzzFolderConfig;
use std::sync::Mutex;
use std::sync::mpsc;
use std::thread;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    path::PathBuf,
};

use fm::{FILE_EXTENSION, FileManager};
use noirc_driver::{add_dep, prepare_crate, prepare_dependency};
use noirc_frontend::{
    graph::{CrateId, CrateName},
    hir::{Context, ParsedFiles, def_map::parse_file},
};
use package::{Dependency, Package};
use walkdir::WalkDir;

pub fn prepare_dependencies(
    context: &mut Context,
    parent_crate: CrateId,
    dependencies: &BTreeMap<CrateName, Dependency>,
) {
    for (dep_name, dep) in dependencies.iter() {
        match dep {
            Dependency::Remote { package } | Dependency::Local { package } => {
                let crate_id = prepare_dependency(context, &package.entry_path);
                add_unstable_features(context, crate_id, package);
                add_dep(context, parent_crate, crate_id, dep_name.clone());
                prepare_dependencies(context, crate_id, &package.dependencies);
            }
        }
    }
}

// We will pre-populate the file manager with all the files in the package
// This is so that we can avoid having to read from disk when we are compiling
//
// This does not require parsing because we are interested in the files under the src directory
// it may turn out that we do not need to include some Noir files that we add to the file
// manager

pub fn insert_all_files_for_workspace_into_file_manager(
    workspace: &workspace::Workspace,
    file_manager: &mut FileManager,
) {
    insert_all_files_for_workspace_into_file_manager_with_overrides(workspace, file_manager, None);
}

pub fn insert_all_files_for_workspace_into_file_manager_with_overrides(
    workspace: &workspace::Workspace,
    file_manager: &mut FileManager,
    overrides: Option<&HashMap<PathBuf, &str>>,
) {
    let mut processed_entry_paths = HashSet::new();

    // We first collect all files, then add the overrides, and sort all of them
    // so we always get a consistent order of the files, even if an override
    // doesn't exist in the filesystem.
    let mut filenames = Vec::new();
    let mut seen_filenames = HashSet::new();
    for package in workspace.clone().into_iter() {
        collect_all_files_in_package(
            package,
            &mut filenames,
            &mut seen_filenames,
            &mut processed_entry_paths,
        );
    }
    if let Some(overrides) = overrides {
        filenames.extend(overrides.keys().cloned());
    }

    insert_all_files_into_file_manager(file_manager, overrides, filenames);
}

pub fn insert_all_files_under_path(
    file_manager: &mut FileManager,
    path: &std::path::Path,
    overrides: Option<&HashMap<PathBuf, &str>>,
) {
    let mut filenames = Vec::new();
    let mut seen_filenames = HashSet::new();
    collect_all_files_under_path(path, &mut filenames, &mut seen_filenames);

    if let Some(overrides) = overrides {
        for override_name in overrides.keys() {
            if seen_filenames.insert(override_name.clone()) {
                filenames.push(override_name.clone());
            }
        }
    }

    // Overrides can only happen in an LSP session. In that case we need to sort
    // all filenames for a consistent order.
    // Outside of LSP there are no overrides and the order given by the filesystem
    // is good and consistent across machines.
    if overrides.is_some() {
        filenames.sort();
    }

    insert_all_files_into_file_manager(file_manager, overrides, filenames);
}

fn insert_all_files_into_file_manager(
    file_manager: &mut FileManager,
    overrides: Option<&HashMap<PathBuf, &str>>,
    filenames: Vec<PathBuf>,
) {
    for filename in filenames {
        let source = if let Some(src) = overrides.and_then(|overrides| overrides.get(&filename)) {
            src.to_string()
        } else {
            std::fs::read_to_string(filename.as_path())
                .unwrap_or_else(|_| panic!("could not read file {filename:?} into string"))
        };

        file_manager.add_file_with_source(filename.as_path(), source);
    }
}

fn collect_all_files_in_package(
    package: &Package,
    filenames: &mut Vec<PathBuf>,
    seen_filenames: &mut HashSet<PathBuf>,
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

    collect_all_files_under_path(entry_path_parent, filenames, seen_filenames);

    collect_all_files_in_packages_dependencies(
        package,
        filenames,
        seen_filenames,
        processed_entry_paths,
    );
}

// Collect all files for the dependencies of the package too
fn collect_all_files_in_packages_dependencies(
    package: &Package,
    filenames: &mut Vec<PathBuf>,
    seen_filenames: &mut HashSet<PathBuf>,
    processed_entry_paths: &mut HashSet<PathBuf>,
) {
    for (_, dep) in package.dependencies.iter() {
        match dep {
            Dependency::Local { package } | Dependency::Remote { package } => {
                collect_all_files_in_package(
                    package,
                    filenames,
                    seen_filenames,
                    processed_entry_paths,
                );
            }
        }
    }
}

fn collect_all_files_under_path(
    path: &std::path::Path,
    filenames: &mut Vec<PathBuf>,
    seen_filenames: &mut HashSet<PathBuf>,
) {
    for entry in WalkDir::new(path).sort_by_file_name() {
        let Ok(entry) = entry else {
            continue;
        };

        if !entry.file_type().is_file() {
            continue;
        }

        if entry.path().extension().is_none_or(|ext| ext != FILE_EXTENSION) {
            continue;
        };

        let path = entry.into_path();
        if seen_filenames.insert(path.clone()) {
            filenames.push(path);
        }
    }
}

const STACK_SIZE: usize = 8 * 1024 * 1024;

#[cfg(any(target_arch = "wasm32", target_arch = "wasm64"))]
pub fn parse_all(file_manager: &FileManager) -> ParsedFiles {
    use rayon::iter::ParallelBridge as _;
    use rayon::iter::ParallelIterator as _;

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

#[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
pub fn parse_all(file_manager: &FileManager) -> ParsedFiles {
    // Collect only .nr files to process
    let nr_files: Vec<_> = file_manager
        .as_file_map()
        .all_file_ids()
        .filter(|&&file_id| {
            let file_path = file_manager.path(file_id).expect("expected file to exist");
            let file_extension =
                file_path.extension().expect("expected all file paths to have an extension");
            file_extension == "nr"
        })
        .copied()
        .collect();

    // Limit threads to the actual number of files we need to process.
    let num_threads = rayon::current_num_threads().min(nr_files.len()).max(1);

    let (sender, receiver) = mpsc::channel();
    let iter = &Mutex::new(nr_files.into_iter());

    thread::scope(|scope| {
        // Start worker threads
        for _ in 0..num_threads {
            // Clone sender so it's dropped once the thread finishes
            let thread_sender = sender.clone();
            thread::Builder::new()
                // Specify a larger-than-default stack size to prevent overflowing stack in large programs.
                // (the default is 2MB)
                .stack_size(STACK_SIZE)
                .spawn_scoped(scope, move || {
                    loop {
                        // Get next file to process from the iterator.
                        let Some(file_id) = iter.lock().unwrap().next() else {
                            break;
                        };

                        let parsed_file = parse_file(file_manager, file_id);

                        if thread_sender.send((file_id, parsed_file)).is_err() {
                            break;
                        }
                    }
                })
                .unwrap();
        }

        // Also drop main sender so the channel closes
        drop(sender);

        let mut parsed_files = ParsedFiles::default();
        while let Ok((file_id, parsed_file)) = receiver.recv() {
            parsed_files.insert(file_id, parsed_file);
        }

        parsed_files
    })
}

#[tracing::instrument(level = "trace", skip_all)]
pub fn prepare_package<'file_manager, 'parsed_files>(
    file_manager: &'file_manager FileManager,
    parsed_files: &'parsed_files ParsedFiles,
    package: &Package,
) -> (Context<'file_manager, 'parsed_files>, CrateId) {
    let mut context = Context::from_ref_file_manager(file_manager, parsed_files);
    let crate_id = prepare_crate(&mut context, &package.entry_path);
    add_unstable_features(&mut context, crate_id, package);
    prepare_dependencies(&mut context, crate_id, &package.dependencies);
    (context, crate_id)
}

/// Add any unstable features required by the `Package` to the `Context`.
fn add_unstable_features(context: &mut Context, crate_id: CrateId, package: &Package) {
    context
        .required_unstable_features
        .insert(crate_id, package.compiler_required_unstable_features.clone());
}
