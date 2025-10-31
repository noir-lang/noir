use std::collections::{HashMap, HashSet};

use crate::items::{Item, TraitImpl, TypeId, Workspace};

/// Gather all trait impls in the workspace, grouped by the trait they implement.
pub(super) fn gather_all_trait_impls(workspace: &Workspace) -> HashMap<TypeId, HashSet<TraitImpl>> {
    let mut trait_impls = HashMap::new();

    for krate in &workspace.crates {
        for item in &krate.root_module.items {
            gather_trait_impls_in_item(item, &mut trait_impls);
        }
    }

    trait_impls
}

fn gather_trait_impls_in_item(item: &Item, trait_impls: &mut HashMap<TypeId, HashSet<TraitImpl>>) {
    match item {
        Item::Module(module) => {
            for item in &module.items {
                gather_trait_impls_in_item(item, trait_impls);
            }
        }
        Item::Struct(struct_) => {
            for impl_ in &struct_.trait_impls {
                trait_impls.entry(impl_.trait_id).or_default().insert(impl_.clone());
            }
        }
        Item::Trait(trait_) => {
            for impl_ in &trait_.trait_impls {
                trait_impls.entry(impl_.trait_id).or_default().insert(impl_.clone());
            }
        }
        Item::TypeAlias(_) | Item::Function(_) | Item::Global(_) => {}
    }
}
