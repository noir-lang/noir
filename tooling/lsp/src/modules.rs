use std::collections::BTreeMap;

use noirc_frontend::{
    ast::Ident,
    graph::{CrateId, Dependency},
    hir::def_map::{CrateDefMap, ModuleDefId, ModuleId},
    node_interner::{NodeInterner, Reexport, ReferenceId},
};

use crate::visibility::module_def_id_is_visible;

pub(crate) fn get_parent_module(
    interner: &NodeInterner,
    module_def_id: ModuleDefId,
) -> Option<ModuleId> {
    let reference_id = module_def_id_to_reference_id(module_def_id);
    interner.reference_module(reference_id).copied()
}

pub(crate) fn module_def_id_to_reference_id(module_def_id: ModuleDefId) -> ReferenceId {
    match module_def_id {
        ModuleDefId::ModuleId(id) => ReferenceId::Module(id),
        ModuleDefId::FunctionId(id) => ReferenceId::Function(id),
        ModuleDefId::TypeId(id) => ReferenceId::Type(id),
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
    current_module_id: ModuleId,
    current_module_parent_id: Option<ModuleId>,
    interner: &NodeInterner,
) -> Option<String> {
    let full_path;
    if let ModuleDefId::ModuleId(module_id) = module_def_id {
        full_path = relative_module_id_path(
            module_id,
            current_module_id,
            current_module_parent_id,
            interner,
        );
    } else {
        let parent_module = get_parent_module(interner, module_def_id)?;

        full_path = relative_module_id_path(
            parent_module,
            current_module_id,
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
    current_module_id: ModuleId,
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

            if current_module_id == *parent_module_id {
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

/// Finds a visible reexport for the parent module of the given ModuleDefId.
pub(crate) fn get_parent_module_reexport<'a>(
    module_def_id: ModuleDefId,
    current_module_id: ModuleId,
    interner: &'a NodeInterner,
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
    dependencies: &[Dependency],
) -> Option<&'a Reexport> {
    let parent_module = get_parent_module(interner, module_def_id)?;
    let reexport =
        interner.get_reexports(ModuleDefId::ModuleId(parent_module)).iter().find(|reexport| {
            module_def_id_is_visible(
                ModuleDefId::ModuleId(reexport.module_id),
                current_module_id,
                reexport.visibility,
                None,
                interner,
                def_maps,
                dependencies,
            )
        })?;

    Some(reexport)
}

/// Returns the relative path to reach `module_def_id` named `name` starting from `current_module_id`.
///
/// - `defining_module` might be `Some` if the item is reexported from another module
/// - `intermediate_name` might be `Some` if the item's parent module is reexport from another module
///   (this will be the name of the reexport)
///
/// Returns `None` if `module_def_id` isn't visible from the current module, neither directly, neither via
/// any of its reexports (or parent module reexports).
pub(crate) fn module_def_id_relative_path(
    module_def_id: ModuleDefId,
    name: &str,
    current_module_id: ModuleId,
    current_module_parent_id: Option<ModuleId>,
    defining_module: Option<ModuleId>,
    intermediate_name: &Option<Ident>,
    interner: &NodeInterner,
) -> Option<String> {
    let module_path = if let Some(defining_module) = defining_module {
        relative_module_id_path(
            defining_module,
            current_module_id,
            current_module_parent_id,
            interner,
        )
    } else {
        let Some(module_full_path) = relative_module_full_path(
            module_def_id,
            current_module_id,
            current_module_parent_id,
            interner,
        ) else {
            return None;
        };
        module_full_path
    };

    let path = if defining_module.is_some() || !matches!(module_def_id, ModuleDefId::ModuleId(..)) {
        if let Some(reexport_name) = &intermediate_name {
            format!("{}::{}::{}", module_path, reexport_name, name)
        } else {
            format!("{}::{}", module_path, name)
        }
    } else {
        module_path.clone()
    };

    Some(path)
}
