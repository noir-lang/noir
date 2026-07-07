use std::path::Path;

use noirc_frontend::{
    graph::{CrateId, CrateName},
    hir::Context,
};

use crate::file_manager::DEBUG_CRATE_NAME;

pub(super) const STD_CRATE_NAME: &str = "std";

/// Adds the file from the file system at `Path` to the crate graph as a root file
///
/// Note: If the stdlib dependency has not been added yet, it's added. Otherwise
/// this method assumes the root crate is the stdlib (useful for running tests
/// in the stdlib, getting LSP stuff for the stdlib, etc.).
pub fn prepare_crate(context: &mut Context, file_name: &Path) -> CrateId {
    let path_to_std_lib_file = Path::new(STD_CRATE_NAME).join("lib.nr");
    let std_file_id = context.file_manager.name_to_id(path_to_std_lib_file);
    let std_crate_id = std_file_id.map(|std_file_id| context.crate_graph.add_stdlib(std_file_id));

    let root_file_id = context.file_manager.name_to_id(file_name.to_path_buf()).unwrap_or_else(|| panic!("files are expected to be added to the FileManager before reaching the compiler file_path: {}", file_name.display()));

    if let Some(std_crate_id) = std_crate_id {
        let root_crate_id = context.crate_graph.add_crate_root(root_file_id);

        add_dep(context, root_crate_id, std_crate_id, STD_CRATE_NAME.parse().unwrap());

        root_crate_id
    } else {
        context.crate_graph.add_crate_root_and_stdlib(root_file_id)
    }
}

pub fn link_to_debug_crate(context: &mut Context, root_crate_id: CrateId) {
    let path_to_debug_lib_file = Path::new(DEBUG_CRATE_NAME).join("lib.nr");
    let debug_crate_id = prepare_dependency(context, &path_to_debug_lib_file);
    add_dep(context, root_crate_id, debug_crate_id, DEBUG_CRATE_NAME.parse().unwrap());
    context.debug_crate_id = Some(debug_crate_id);
}

// Adds the file from the file system at `Path` to the crate graph
pub fn prepare_dependency(context: &mut Context, file_name: &Path) -> CrateId {
    let root_file_id = context
        .file_manager
        .name_to_id(file_name.to_path_buf())
        .unwrap_or_else(|| panic!("files are expected to be added to the FileManager before reaching the compiler file_path: {}", file_name.display()));

    let crate_id = context.crate_graph.add_crate(root_file_id);

    // Every dependency has access to stdlib
    let std_crate_id = context.stdlib_crate_id();
    add_dep(context, crate_id, *std_crate_id, STD_CRATE_NAME.parse().unwrap());

    crate_id
}

/// Adds a edge in the crate graph for two crates
pub fn add_dep(
    context: &mut Context,
    this_crate: CrateId,
    depends_on: CrateId,
    crate_name: CrateName,
) {
    context
        .crate_graph
        .add_dep(this_crate, crate_name, depends_on)
        .expect("cyclic dependency triggered");
}
