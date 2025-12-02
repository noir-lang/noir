use crate::Type;
use crate::node_interner::{FuncId, NodeInterner, TraitId, TypeId};

use crate::ast::ItemVisibility;
use crate::hir::def_map::{CrateDefMap, DefMaps, LocalModuleId, ModuleDefId, ModuleId};

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
    def_maps: &DefMaps,
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
            module_is_descendant_of_target(
                target_crate_def_map,
                current_module.local_id,
                target_module.local_id,
            ) || module_is_parent_of_struct_module(
                target_crate_def_map,
                current_module.local_id,
                target_module.local_id,
            )
        }
    }
}

/// Returns true if `current` is a (potentially nested) child module of `target`.
/// This is also true if `current == target`.
fn module_is_descendant_of_target(
    def_map: &CrateDefMap,
    current: LocalModuleId,
    target: LocalModuleId,
) -> bool {
    if current == target {
        return true;
    }

    def_map[current]
        .parent
        .is_some_and(|parent| module_is_descendant_of_target(def_map, parent, target))
}

/// Returns true if `target` is a struct and its parent is `current`.
fn module_is_parent_of_struct_module(
    def_map: &CrateDefMap,
    current: LocalModuleId,
    target: LocalModuleId,
) -> bool {
    let module_data = &def_map[target];
    module_data.is_type && module_data.parent == Some(current)
}

/// Returns whether a struct member with the given visibility is visible from `current_module_id`.
pub fn struct_member_is_visible(
    struct_id: TypeId,
    visibility: ItemVisibility,
    current_module_id: ModuleId,
    def_maps: &DefMaps,
) -> bool {
    type_member_is_visible(struct_id.module_id(), visibility, current_module_id, def_maps)
}

/// Returns whether a trait member with the given visibility is visible from `current_module_id`.
pub fn trait_member_is_visible(
    trait_id: TraitId,
    visibility: ItemVisibility,
    current_module_id: ModuleId,
    def_maps: &DefMaps,
) -> bool {
    type_member_is_visible(trait_id.0, visibility, current_module_id, def_maps)
}

/// Returns whether a struct or trait member with the given visibility is visible from `current_module_id`.
fn type_member_is_visible(
    type_module_id: ModuleId,
    visibility: ItemVisibility,
    current_module_id: ModuleId,
    def_maps: &DefMaps,
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
            module_is_descendant_of_target(
                def_map,
                current_module_id.local_id,
                type_parent_module_id.local_id,
            )
        }
    }
}

/// Returns whether a method call `func_id` on an object of type `object_type` is visible from
/// `current_module`.
/// If there's a self type at the current location it must be passed as `self_type`. This is
/// used for the case of calling, inside a generic trait impl, a private method on the same
/// type as `self_type` regardless of its generic arguments (in this case the call is allowed).
pub fn method_call_is_visible(
    self_type: Option<&Type>,
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

            // A private method defined on `Foo<i32>` should be visible when calling
            // it from an impl on `Foo<i64>`, even though the generics are different.
            if self_type
                .is_some_and(|self_type| is_same_type_regardless_generics(self_type, object_type))
            {
                return true;
            }

            if let Some(struct_id) = func_meta.type_id {
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

/// Returns whether two types are the same disregarding their generic arguments.
fn is_same_type_regardless_generics(type1: &Type, type2: &Type) -> bool {
    if type1 == type2 {
        return true;
    }

    match (type1.follow_bindings(), type2.follow_bindings()) {
        (Type::Array(..), Type::Array(..)) => true,
        (Type::Slice(..), Type::Slice(..)) => true,
        (Type::String(..), Type::String(..)) => true,
        (Type::FmtString(..), Type::FmtString(..)) => true,
        (Type::Tuple(..), Type::Tuple(..)) => true,
        (Type::Function(..), Type::Function(..)) => true,
        (Type::DataType(data_type1, ..), Type::DataType(data_type2, ..)) => {
            data_type1.borrow().id == data_type2.borrow().id
        }
        (Type::Reference(type1, _), _) => is_same_type_regardless_generics(&type1, type2),
        (_, Type::Reference(type2, _)) => is_same_type_regardless_generics(type1, &type2),
        _ => false,
    }
}

pub fn module_def_id_visibility(
    module_def_id: ModuleDefId,
    interner: &NodeInterner,
) -> ItemVisibility {
    match module_def_id {
        ModuleDefId::ModuleId(module_id) => {
            let attributes = interner.try_module_attributes(module_id);
            attributes.map_or(ItemVisibility::Private, |a| a.visibility)
        }
        ModuleDefId::FunctionId(func_id) => interner.function_modifiers(&func_id).visibility,
        ModuleDefId::TypeId(type_id) => {
            let data_type = interner.get_type(type_id);
            data_type.borrow().visibility
        }
        ModuleDefId::TypeAliasId(type_alias_id) => {
            let type_alias = interner.get_type_alias(type_alias_id);
            type_alias.borrow().visibility
        }
        ModuleDefId::TraitAssociatedTypeId(_) => ItemVisibility::Public,
        ModuleDefId::TraitId(trait_id) => {
            let trait_ = interner.get_trait(trait_id);
            trait_.visibility
        }
        ModuleDefId::GlobalId(global_id) => {
            let global_info = interner.get_global(global_id);
            global_info.visibility
        }
    }
}
