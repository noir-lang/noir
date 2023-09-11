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
    graph::{CrateGraph, CrateId, CrateName},
    hir::Context,
};
use package::{Dependency, Package};

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

pub fn prepare_package(package: &Package) -> (Context, CrateId) {
    // TODO: FileManager continues to leak into various crates
    let fm = FileManager::new(&package.root_dir);
    let graph = CrateGraph::default();
    let mut context = Context::new(fm, graph);

    let crate_id = prepare_crate(&mut context, &package.entry_path);

    prepare_dependencies(&mut context, crate_id, &package.dependencies);

    (context, crate_id)
}
