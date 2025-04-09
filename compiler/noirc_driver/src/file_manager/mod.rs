use std::path::Path;

use fm::FileManager;
use noirc_frontend::debug::build_debug_crate_file;

mod stdlib;

pub(super) const DEBUG_CRATE_NAME: &str = "__debug";

/// Helper method to return a file manager instance with the stdlib already added
///
/// TODO: This should become the canonical way to create a file manager and
/// TODO if we use a File manager trait, we can move file manager into this crate
/// TODO as a module
pub fn file_manager_with_stdlib(root: &Path) -> FileManager {
    let mut file_manager = FileManager::new(root);

    add_stdlib_source_to_file_manager(&mut file_manager);
    add_debug_source_to_file_manager(&mut file_manager);

    file_manager
}

/// Adds the source code for the stdlib into the file manager
fn add_stdlib_source_to_file_manager(file_manager: &mut FileManager) {
    // Add the stdlib contents to the file manager, since every package automatically has a dependency
    // on the stdlib. For other dependencies, we read the package.Dependencies file to add their file
    // contents to the file manager. However since the dependency on the stdlib is implicit, we need
    // to manually add it here.
    let stdlib_paths_with_source = stdlib::stdlib_paths_with_source();
    for (path, source) in stdlib_paths_with_source {
        file_manager.add_file_with_source_canonical_path(Path::new(&path), source);
    }
}

/// Adds the source code of the debug crate needed to support instrumentation to
/// track variables values
fn add_debug_source_to_file_manager(file_manager: &mut FileManager) {
    // Adds the synthetic debug module for instrumentation into the file manager
    let path_to_debug_lib_file = Path::new(DEBUG_CRATE_NAME).join("lib.nr");
    file_manager
        .add_file_with_source_canonical_path(&path_to_debug_lib_file, build_debug_crate_file());
}
