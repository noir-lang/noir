use std::collections::BTreeMap;

use noirc_frontend::{
    ast::ItemVisibility,
    graph::{CrateId, Dependency},
    hir::def_map::{CrateDefMap, ModuleId},
    macros_api::{ModuleDefId, NodeInterner},
    node_interner::ReferenceId,
};

use crate::visibility::is_visible;

pub(crate) fn get_parent_module(
    interner: &NodeInterner,
    module_def_id: ModuleDefId,
) -> Option<ModuleId> {
    let reference_id = module_def_id_to_reference_id(module_def_id);
    interner.reference_module(reference_id).copied()
}

pub(crate) fn get_parent_module_id(
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
    module_id: ModuleId,
) -> Option<ModuleId> {
    let crate_def_map = &def_maps[&module_id.krate];
    let module_data = &crate_def_map.modules()[module_id.local_id.0];
    module_data.parent.map(|parent| ModuleId { krate: module_id.krate, local_id: parent })
}

pub(crate) fn module_def_id_to_reference_id(module_def_id: ModuleDefId) -> ReferenceId {
    match module_def_id {
        ModuleDefId::ModuleId(id) => ReferenceId::Module(id),
        ModuleDefId::FunctionId(id) => ReferenceId::Function(id),
        ModuleDefId::TypeId(id) => ReferenceId::Struct(id),
        ModuleDefId::TypeAliasId(id) => ReferenceId::Alias(id),
        ModuleDefId::TraitId(id) => ReferenceId::Trait(id),
        ModuleDefId::GlobalId(id) => ReferenceId::Global(id),
    }
}

/// Returns the fully-qualified path of the given `ModuleDefId` relative to `current_module_id`:
/// - If `ModuleDefId` is a module, that module's path is returned
/// - Otherwise, that item's parent module's path is returned
pub(crate) fn relative_module_full_path(
    module_def_id: ModuleDefId,
    visibility: ItemVisibility,
    current_module_id: ModuleId,
    current_module_parent_id: Option<ModuleId>,
    interner: &NodeInterner,
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
) -> Option<String> {
    let full_path;
    if let ModuleDefId::ModuleId(module_id) = module_def_id {
        if !is_visible(module_id, current_module_id, visibility, def_maps) {
            return None;
        }

        full_path = relative_module_id_path(
            module_id,
            &current_module_id,
            current_module_parent_id,
            interner,
        );
    } else {
        let Some(parent_module) = get_parent_module(interner, module_def_id) else {
            return None;
        };

        if !is_visible(parent_module, current_module_id, visibility, def_maps) {
            return None;
        }

        full_path = relative_module_id_path(
            parent_module,
            &current_module_id,
            current_module_parent_id,
            interner,
        );
    }
    Some(full_path)
}

/// Returns the path to reach an item inside `target_module_id` from inside `current_module_id`.
/// Returns a relative path if possible.
pub(crate) fn relative_module_id_path(
    target_module_id: ModuleId,
    current_module_id: &ModuleId,
    current_module_parent_id: Option<ModuleId>,
    interner: &NodeInterner,
) -> String {
    if Some(target_module_id) == current_module_parent_id {
        return "super".to_string();
    }

    let mut segments: Vec<&str> = Vec::new();
    let mut is_relative = false;

    if let Some(module_attributes) = interner.try_module_attributes(&target_module_id) {
        segments.push(&module_attributes.name);

        let mut current_attributes = module_attributes;
        loop {
            let Some(parent_local_id) = current_attributes.parent else {
                break;
            };

            let parent_module_id =
                &ModuleId { krate: target_module_id.krate, local_id: parent_local_id };

            if current_module_id == parent_module_id {
                is_relative = true;
                break;
            }

            if current_module_parent_id == Some(*parent_module_id) {
                segments.push("super");
                is_relative = true;
                break;
            }

            let Some(parent_attributes) = interner.try_module_attributes(parent_module_id) else {
                break;
            };

            segments.push(&parent_attributes.name);
            current_attributes = parent_attributes;
        }
    }

    if !is_relative {
        // We don't record module attributes for the root module,
        // so we handle that case separately
        if target_module_id.krate.is_root() {
            segments.push("crate");
        }
    }

    segments.reverse();
    segments.join("::")
}

pub(crate) fn module_full_path(
    module: &ModuleId,
    interner: &NodeInterner,
    crate_id: CrateId,
    crate_name: &str,
    dependencies: &Vec<Dependency>,
) -> String {
    let mut segments: Vec<String> = Vec::new();

    if let Some(module_attributes) = interner.try_module_attributes(module) {
        segments.push(module_attributes.name.clone());

        let mut current_attributes = module_attributes;
        loop {
            let Some(parent_local_id) = current_attributes.parent else {
                break;
            };

            let Some(parent_attributes) = interner.try_module_attributes(&ModuleId {
                krate: module.krate,
                local_id: parent_local_id,
            }) else {
                break;
            };

            segments.push(parent_attributes.name.clone());
            current_attributes = parent_attributes;
        }
    }

    // We don't record module attributes for the root module,
    // so we handle that case separately
    if module.krate.is_root() {
        if module.krate == crate_id {
            segments.push(crate_name.to_string());
        } else {
            for dep in dependencies {
                if dep.crate_id == crate_id {
                    segments.push(dep.name.to_string());
                    break;
                }
            }
        }
    };

    segments.reverse();
    segments.join("::")
}
