#![forbid(unsafe_code)]
#![warn(unused_extern_crates)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]

//! Nargo is the package manager for Noir
//! This name was used because it sounds like `cargo` and
//! Noir Package Manager abbreviated is npm, which is already taken.

use fm::FileManager;
use nargo::package::{Dependency, Package};
use noirc_driver::{add_dep, prepare_crate};
use noirc_frontend::{
    graph::{CrateGraph, CrateId, CrateName},
    hir::Context,
};
use std::collections::BTreeMap;

mod backends;
pub mod cli;
mod errors;

fn prepare_dependencies(
    context: &mut Context,
    parent_crate: CrateId,
    dependencies: &BTreeMap<CrateName, Dependency>,
) {
    for (dep_name, dep) in dependencies.iter() {
        match dep {
            Dependency::Remote { package } | Dependency::Local { package } => {
                let crate_id = prepare_crate(context, &package.entry_path);
                add_dep(context, parent_crate, crate_id, dep_name.clone());
                prepare_dependencies(context, crate_id, &package.dependencies);
            }
        }
    }
}

fn prepare_package(package: &Package) -> (Context, CrateId) {
    let fm = FileManager::new(&package.root_dir);
    let graph = CrateGraph::default();
    let mut context = Context::new(fm, graph);

    let crate_id = prepare_crate(&mut context, &package.entry_path);

    prepare_dependencies(&mut context, crate_id, &package.dependencies);

    (context, crate_id)
}
