use std::collections::HashMap;

use noirc_frontend::ast::ItemVisibility;

use crate::{
    html::{HasClass, has_uri::HasUri},
    items::{Item, ItemId, Workspace},
};

pub(super) struct ItemInfo {
    pub(super) path: Vec<String>,
    pub(super) uri: String,
    pub(super) class: &'static str,
    /// Overall visibility of this item, including parent modules.
    /// For example, if this is a public item inside a private module,
    /// its visibility will be private.
    pub(super) visibility: ItemVisibility,
}

/// Computes an ItemInfo for every item in the workspace, indexed by its Id.
pub(super) fn compute_id_to_info(workspace: &Workspace) -> HashMap<ItemId, ItemInfo> {
    let mut id_to_info = HashMap::new();
    let mut path = Vec::new();

    for krate in workspace.all_crates() {
        let module = &krate.root_module;
        let uri = krate.uri();
        let class = module.class();
        let visibility = ItemVisibility::Public;
        id_to_info.insert(module.id.clone(), ItemInfo { path: Vec::new(), uri, class, visibility });

        path.push(krate.name.to_string());

        for (visibility, item) in &krate.root_module.items {
            compute_id_to_info_in_item(item, *visibility, &mut id_to_info, &mut path);
        }

        path.pop();
    }

    id_to_info
}

fn compute_id_to_info_in_item(
    item: &Item,
    visibility: ItemVisibility,
    id_to_info: &mut HashMap<ItemId, ItemInfo>,
    path: &mut Vec<String>,
) {
    match item {
        Item::Module(module) => {
            let uri = format!("{}/{}", path.join("/"), module.uri());
            let class = module.class();
            id_to_info
                .insert(module.id.clone(), ItemInfo { path: path.clone(), uri, class, visibility });

            path.push(module.name.clone());
            for (item_visibility, item) in &module.items {
                let visibility = visibility.min(*item_visibility);
                compute_id_to_info_in_item(item, visibility, id_to_info, path);
            }
            path.pop();
        }
        Item::Struct(struct_) => {
            let uri = format!("{}/{}", path.join("/"), struct_.uri());
            let class = struct_.class();
            id_to_info.insert(
                struct_.id.clone(),
                ItemInfo { path: path.clone(), uri, class, visibility },
            );
        }
        Item::Trait(trait_) => {
            let uri = format!("{}/{}", path.join("/"), trait_.uri());
            let class = trait_.class();
            id_to_info
                .insert(trait_.id.clone(), ItemInfo { path: path.clone(), uri, class, visibility });
        }
        Item::TypeAlias(type_alias) => {
            let uri = format!("{}/{}", path.join("/"), type_alias.uri());
            let class = type_alias.class();
            id_to_info.insert(
                type_alias.id.clone(),
                ItemInfo { path: path.clone(), uri, class, visibility },
            );
        }
        Item::Function(function) => {
            let uri = format!("{}/{}", path.join("/"), function.uri());
            let class = function.class();
            id_to_info.insert(
                function.id.clone(),
                ItemInfo { path: path.clone(), uri, class, visibility },
            );
        }
        Item::Global(global) => {
            let uri = format!("{}/{}", path.join("/"), global.uri());
            let class = global.class();
            id_to_info
                .insert(global.id.clone(), ItemInfo { path: path.clone(), uri, class, visibility });
        }
        Item::PrimitiveType(_) | Item::Reexport(_) => {}
    }
}
