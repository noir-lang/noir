use crate::graph::CrateId;
use crate::node_interner::{FuncId, NodeInterner, StructId, TraitId};
use crate::Type;

use std::collections::BTreeMap;

use crate::ast::ItemVisibility;
use crate::hir::def_map::{CrateDefMap, DefMaps, LocalModuleId, ModuleId};

/// Returns true if an item with the given visibility in the target module
/// is visible from the current module. For example:
/// ```text
/// mod foo {
///     ^^^ <-- target module
///   pub(crate) fn bar() {}
///   ^^^^^^^^^^ <- visibility
/// }
/// ```
pub fn item_in_module_is_visible(
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
    current_module: ModuleId,
    target_module: ModuleId,
    visibility: ItemVisibility,
) -> bool {
    // Note that if the target module is in a different crate from the current module then we will either
    // return true as the target module is public or return false as it is private without looking at the `CrateDefMap` in either case.
    let same_crate = target_module.krate == current_module.krate;

    match visibility {
        ItemVisibility::Public => true,
        ItemVisibility::PublicCrate => same_crate,
        ItemVisibility::Private => {
            if !same_crate {
                return false;
            }

            let target_crate_def_map = &def_maps[&target_module.krate];
            module_descendent_of_target(
                target_crate_def_map,
                target_module.local_id,
                current_module.local_id,
            ) || module_is_parent_of_struct_module(
                target_crate_def_map,
                current_module.local_id,
                target_module.local_id,
            )
        }
    }
}

// Returns true if `current` is a (potentially nested) child module of `target`.
// This is also true if `current == target`.
pub(crate) fn module_descendent_of_target(
    def_map: &CrateDefMap,
    target: LocalModuleId,
    current: LocalModuleId,
) -> bool {
    if current == target {
        return true;
    }

    def_map.modules[current.0]
        .parent
        .map_or(false, |parent| module_descendent_of_target(def_map, target, parent))
}

/// Returns true if `target` is a struct and its parent is `current`.
fn module_is_parent_of_struct_module(
    def_map: &CrateDefMap,
    current: LocalModuleId,
    target: LocalModuleId,
) -> bool {
    let module_data = &def_map.modules[target.0];
    module_data.is_struct && module_data.parent == Some(current)
}

pub fn struct_member_is_visible(
    struct_id: StructId,
    visibility: ItemVisibility,
    current_module_id: ModuleId,
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
) -> bool {
    type_member_is_visible(struct_id.module_id(), visibility, current_module_id, def_maps)
}

pub fn trait_member_is_visible(
    trait_id: TraitId,
    visibility: ItemVisibility,
    current_module_id: ModuleId,
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
) -> bool {
    type_member_is_visible(trait_id.0, visibility, current_module_id, def_maps)
}

fn type_member_is_visible(
    type_module_id: ModuleId,
    visibility: ItemVisibility,
    current_module_id: ModuleId,
    def_maps: &BTreeMap<CrateId, CrateDefMap>,
) -> bool {
    match visibility {
        ItemVisibility::Public => true,
        ItemVisibility::PublicCrate => {
            let type_parent_module_id =
                type_module_id.parent(def_maps).expect("Expected parent module to exist");
            type_parent_module_id.krate == current_module_id.krate
        }
        ItemVisibility::Private => {
            let type_parent_module_id =
                type_module_id.parent(def_maps).expect("Expected parent module to exist");
            if type_parent_module_id.krate != current_module_id.krate {
                return false;
            }

            if type_parent_module_id.local_id == current_module_id.local_id {
                return true;
            }

            let def_map = &def_maps[&current_module_id.krate];
            module_descendent_of_target(
                def_map,
                type_parent_module_id.local_id,
                current_module_id.local_id,
            )
        }
    }
}

pub fn method_call_is_visible(
    object_type: &Type,
    func_id: FuncId,
    current_module: ModuleId,
    interner: &NodeInterner,
    def_maps: &DefMaps,
) -> bool {
    let modifiers = interner.function_modifiers(&func_id);
    match modifiers.visibility {
        ItemVisibility::Public => true,
        ItemVisibility::PublicCrate | ItemVisibility::Private => {
            let func_meta = interner.function_meta(&func_id);

            if let Some(trait_id) = func_meta.trait_id {
                return trait_member_is_visible(
                    trait_id,
                    modifiers.visibility,
                    current_module,
                    def_maps,
                );
            }

            if let Some(trait_impl_id) = func_meta.trait_impl {
                let trait_impl = interner.get_trait_implementation(trait_impl_id);
                return trait_member_is_visible(
                    trait_impl.borrow().trait_id,
                    modifiers.visibility,
                    current_module,
                    def_maps,
                );
            }

            if let Some(struct_id) = func_meta.struct_id {
                return struct_member_is_visible(
                    struct_id,
                    modifiers.visibility,
                    current_module,
                    def_maps,
                );
            }

            if object_type.is_primitive() {
                let func_module = interner.function_module(func_id);
                return item_in_module_is_visible(
                    def_maps,
                    current_module,
                    func_module,
                    modifiers.visibility,
                );
            }

            true
        }
    }
}
