use std::collections::{HashMap, HashSet};

use noirc_frontend::ast::ItemVisibility;

use crate::items::{Item, ItemId, TraitImpl, Workspace};

/// Gather all trait impls in the workspace, grouped by the trait they implement.
pub(super) fn gather_all_trait_impls(workspace: &Workspace) -> HashMap<ItemId, HashSet<TraitImpl>> {
    let mut trait_impls = HashMap::new();

    for krate in workspace.all_crates() {
        for (visibility, item) in &krate.root_module.items {
            if visibility == &ItemVisibility::Public {
                gather_trait_impls_in_item(item, &mut trait_impls);
            }
        }
    }

    trait_impls
}

fn gather_trait_impls_in_item(item: &Item, trait_impls: &mut HashMap<ItemId, HashSet<TraitImpl>>) {
    match item {
        Item::Module(module) => {
            for (visibility, item) in &module.items {
                if visibility == &ItemVisibility::Public {
                    gather_trait_impls_in_item(item, trait_impls);
                }
            }
        }
        Item::Struct(struct_) => {
            for impl_ in &struct_.trait_impls {
                trait_impls.entry(impl_.trait_id.clone()).or_default().insert(impl_.clone());
            }
        }
        Item::Trait(trait_) => {
            for impl_ in &trait_.trait_impls {
                trait_impls.entry(impl_.trait_id.clone()).or_default().insert(impl_.clone());
            }
        }
        Item::PrimitiveType(primitive_type) => {
            for impl_ in &primitive_type.trait_impls {
                trait_impls.entry(impl_.trait_id.clone()).or_default().insert(impl_.clone());
            }
        }
        Item::TypeAlias(_) | Item::Function(_) | Item::Global(_) | Item::Reexport(_) => {}
    }
}
