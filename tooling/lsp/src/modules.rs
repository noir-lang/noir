//! Paths to modules and items as a user would write them from a given module: the `use` path
//! for auto-import, the qualified name shown on hover, the path in a generated trait impl stub.

use noirc_frontend::{
    ast::Ident,
    graph::{CrateId, Dependency},
    hir::def_map::{DefMaps, ModuleDefId, ModuleId},
    modules::get_parent_module,
    node_interner::NodeInterner,
};

/// Returns the fully-qualified path of the given `ModuleDefId` relative to `current_module_id`:
/// - If `ModuleDefId` is a module, that module's path is returned
/// - Otherwise, that item's parent module's path is returned
pub(crate) fn relative_module_full_path(
    module_def_id: ModuleDefId,
    current_module_id: ModuleId,
    current_module_parent_id: Option<ModuleId>,
    interner: &NodeInterner,
    def_maps: &DefMaps,
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
        let parent_module = get_parent_module(module_def_id, interner, def_maps)?;

        // If module_def_id is contained in the current module, the relative path is empty
        if current_module_id == parent_module {
            return None;
        }

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

    // A crate's root module is named after the crate when it's reached from a crate that depends on
    // it, but from inside the crate itself it is always `crate`.
    let same_crate = target_module_id.krate == current_module_id.krate;

    match interner.try_module_attributes(target_module_id) {
        Some(module_attributes) if module_attributes.parent.is_none() => {
            if !same_crate {
                segments.push(&module_attributes.name);
            }
        }
        Some(module_attributes) => {
            segments.push(&module_attributes.name);

            let mut current_attributes = module_attributes;
            while let Some(parent_local_id) = current_attributes.parent {
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

                let Some(parent_attributes) = interner.try_module_attributes(*parent_module_id)
                else {
                    break;
                };

                if parent_attributes.parent.is_none() && same_crate {
                    break;
                }

                segments.push(&parent_attributes.name);
                current_attributes = parent_attributes;
            }
        }
        None => {}
    }

    if !is_relative && same_crate {
        segments.push("crate");
    }

    segments.reverse();
    segments.join("::")
}

pub(crate) fn module_full_path(
    module: ModuleId,
    interner: &NodeInterner,
    crate_id: CrateId,
    crate_name: &str,
    dependencies: &Vec<Dependency>,
) -> String {
    let mut segments: Vec<String> = Vec::new();

    if let Some(module_attributes) = interner.try_module_attributes(module) {
        segments.push(module_attributes.name.clone());

        let mut current_attributes = module_attributes;
        while let Some(parent_local_id) = current_attributes.parent {
            let Some(parent_attributes) = interner
                .try_module_attributes(ModuleId { krate: module.krate, local_id: parent_local_id })
            else {
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
    }

    segments.reverse();
    segments.join("::")
}

/// Returns the relative path to reach `module_def_id` named `name` starting from `current_module_id`.
///
/// - `defining_module` might be `Some` if the item is reexported from another module
/// - `intermediate_name` might be `Some` if the item's parent module is reexport from another module
///   (this will be the name of the reexport)
///
/// Returns `None` if `module_def_id` isn't visible from the current module, neither directly, neither via
/// any of its reexports (or parent module reexports).
#[allow(clippy::too_many_arguments)]
pub(crate) fn module_def_id_relative_path(
    module_def_id: ModuleDefId,
    name: &str,
    current_module_id: ModuleId,
    current_module_parent_id: Option<ModuleId>,
    defining_module: Option<ModuleId>,
    intermediate_name: &Option<Ident>,
    interner: &NodeInterner,
    def_maps: &DefMaps,
) -> Option<String> {
    let module_path = if let Some(defining_module) = defining_module {
        relative_module_id_path(
            defining_module,
            current_module_id,
            current_module_parent_id,
            interner,
        )
    } else {
        relative_module_full_path(
            module_def_id,
            current_module_id,
            current_module_parent_id,
            interner,
            def_maps,
        )?
    };

    let path = if defining_module.is_some() || !matches!(module_def_id, ModuleDefId::ModuleId(..)) {
        if let Some(reexport_name) = &intermediate_name {
            format!("{module_path}::{reexport_name}::{name}")
        } else {
            format!("{module_path}::{name}")
        }
    } else {
        module_path
    };

    Some(path)
}
