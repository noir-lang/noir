use std::collections::HashMap;

use crate::{
    html::has_path::HasPath,
    items::{Item, TypeId, Workspace},
};

pub(super) fn compute_id_to_path(workspace: &Workspace) -> HashMap<TypeId, String> {
    let mut id_to_path = HashMap::new();
    let mut path = Vec::new();

    for krate in &workspace.crates {
        path.push(krate.name.to_string());
        for item in &krate.root_module.items {
            compute_id_to_path_in_item(item, &mut id_to_path, &mut path);
        }

        path.pop();
    }

    id_to_path
}

fn compute_id_to_path_in_item(
    item: &Item,
    id_to_path: &mut HashMap<TypeId, String>,
    path: &mut Vec<String>,
) {
    match item {
        Item::Module(module) => {
            path.push(module.name.clone());
            for item in &module.items {
                compute_id_to_path_in_item(item, id_to_path, path);
            }
            path.pop();
        }
        Item::Struct(struct_) => {
            let path = format!("{}/{}", path.join("/"), struct_.path());
            id_to_path.insert(struct_.id, path);
        }
        Item::Trait(trait_) => {
            let path = format!("{}/{}", path.join("/"), trait_.path());
            id_to_path.insert(trait_.id, path);
        }
        Item::TypeAlias(type_alias) => {
            let path = format!("{}/{}", path.join("/"), type_alias.path());
            id_to_path.insert(type_alias.id, path);
        }
        Item::Function(_) | Item::Global(_) => {}
    }
}
