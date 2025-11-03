use std::collections::HashMap;

use noirc_frontend::ast::ItemVisibility;

use crate::{
    html::has_uri::HasUri,
    items::{Item, TypeId, Workspace},
};

/// Computes the full URI of every type (struct, trait or type alias) in the workspace.
pub(super) fn compute_id_to_uri(workspace: &Workspace) -> HashMap<TypeId, String> {
    let mut id_to_path = HashMap::new();
    let mut path = Vec::new();

    for krate in workspace.all_crates() {
        path.push(krate.name.to_string());
        for (visibility, item) in &krate.root_module.items {
            if visibility == &ItemVisibility::Public {
                compute_id_to_uri_in_item(item, &mut id_to_path, &mut path);
            }
        }

        path.pop();
    }

    id_to_path
}

fn compute_id_to_uri_in_item(
    item: &Item,
    id_to_path: &mut HashMap<TypeId, String>,
    path: &mut Vec<String>,
) {
    match item {
        Item::Module(module) => {
            path.push(module.name.clone());
            for (visibility, item) in &module.items {
                if visibility == &ItemVisibility::Public {
                    compute_id_to_uri_in_item(item, id_to_path, path);
                }
            }
            path.pop();
        }
        Item::Struct(struct_) => {
            let path = format!("{}/{}", path.join("/"), struct_.uri());
            id_to_path.insert(struct_.id, path);
        }
        Item::Trait(trait_) => {
            let path = format!("{}/{}", path.join("/"), trait_.uri());
            id_to_path.insert(trait_.id, path);
        }
        Item::TypeAlias(type_alias) => {
            let path = format!("{}/{}", path.join("/"), type_alias.uri());
            id_to_path.insert(type_alias.id, path);
        }
        Item::Function(_) | Item::Global(_) | Item::PrimitiveType(_) => {}
    }
}
