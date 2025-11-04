use noirc_frontend::ast::ItemVisibility;

use crate::items::{Item, Workspace};

/// All items in a workspace.
/// Each field is a list of (path, name) tuples.
pub(super) struct AllItems {
    pub(super) structs: Vec<(Vec<String>, String)>,
    pub(super) traits: Vec<(Vec<String>, String)>,
    pub(super) type_aliases: Vec<(Vec<String>, String)>,
    pub(super) primitive_types: Vec<(Vec<String>, String)>,
    pub(super) functions: Vec<(Vec<String>, String)>,
    pub(super) globals: Vec<(Vec<String>, String)>,
}

pub(super) fn compute_all_items(workspace: &Workspace) -> AllItems {
    let mut all_items = AllItems {
        structs: Vec::new(),
        traits: Vec::new(),
        type_aliases: Vec::new(),
        primitive_types: Vec::new(),
        globals: Vec::new(),
        functions: Vec::new(),
    };

    let mut current_path = Vec::new();
    for krate in &workspace.crates {
        current_path.push(krate.name.clone());
        for (visibility, item) in &krate.root_module.items {
            if visibility == &ItemVisibility::Public {
                gather_all_items_in_item(item, &mut current_path, &mut all_items);
            }
        }
        current_path.pop();
    }

    all_items.structs.sort();
    all_items.traits.sort();
    all_items.type_aliases.sort();
    all_items.primitive_types.sort();
    all_items.functions.sort();
    all_items.globals.sort();

    all_items
}

fn gather_all_items_in_item(item: &Item, current_path: &mut Vec<String>, all_items: &mut AllItems) {
    match item {
        Item::Module(module) => {
            current_path.push(module.name.clone());
            for (visibility, item) in &module.items {
                if visibility == &ItemVisibility::Public {
                    gather_all_items_in_item(item, current_path, all_items);
                }
            }
            current_path.pop();
        }
        Item::Struct(struct_) => {
            all_items.structs.push((current_path.clone(), struct_.name.clone()));
        }
        Item::Trait(trait_) => {
            all_items.traits.push((current_path.clone(), trait_.name.clone()));
        }
        Item::TypeAlias(type_alias) => {
            all_items.type_aliases.push((current_path.clone(), type_alias.name.clone()));
        }
        Item::PrimitiveType(primitive_type) => {
            all_items.primitive_types.push((current_path.clone(), primitive_type.kind.to_string()));
        }
        Item::Global(global) => {
            all_items.globals.push((current_path.clone(), global.name.clone()));
        }
        Item::Function(function) => {
            all_items.functions.push((current_path.clone(), function.name.clone()));
        }
        Item::Reexport(_) => {}
    }
}
