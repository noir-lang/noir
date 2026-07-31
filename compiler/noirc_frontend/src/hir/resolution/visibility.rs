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

/// Returns true if `module` and every one of its ancestor modules (up to, but not including,
/// the crate root) are visible from `current_module`.
///
/// This mimics the behavior of path resolution, which checks the visibility of each module
/// segment as it walks a path.
pub fn module_is_visible(
    module: ModuleId,
    current_module: ModuleId,
    interner: &NodeInterner,
    def_maps: &DefMaps,
) -> bool {
    // Each module's visibility is declared in its parent, so we check it against the parent just
    // as path resolution checks a segment's visibility against the module it lives in. The crate
    // root has no parent and is always reachable (it is entered via the extern prelude).
    let mut module = module;
    while let Some(parent) = module.parent(def_maps) {
        let visibility = module_def_id_visibility(ModuleDefId::ModuleId(module), interner);
        if !item_in_module_is_visible(def_maps, current_module, parent, visibility) {
            return false;
        }
        module = parent;
    }
    true
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

/// Returns true if the trait that `func_id` belongs to (as a trait declaration or trait impl)
/// is visible from `current_module`. Returns true if `func_id` is not a trait method, or if
/// its function meta has not been registered yet (in which case there's nothing to check).
pub fn trait_visibility_for_method_is_satisfied(
    func_id: FuncId,
    current_module: ModuleId,
    interner: &NodeInterner,
    def_maps: &DefMaps,
) -> bool {
    let Some(func_meta) = interner.try_function_meta(&func_id) else { return true };
    let visibility = interner.function_modifiers(&func_id).visibility;

    if let Some(trait_id) = func_meta.trait_id {
        return trait_member_is_visible(trait_id, visibility, current_module, def_maps);
    }
    if let Some(trait_impl_id) = func_meta.trait_impl {
        let trait_id = interner.get_trait_implementation(trait_impl_id).borrow().trait_id;
        return trait_member_is_visible(trait_id, visibility, current_module, def_maps);
    }
    true
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
///
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

            if func_meta.trait_id.is_some() || func_meta.trait_impl.is_some() {
                return trait_visibility_for_method_is_satisfied(
                    func_id,
                    current_module,
                    interner,
                    def_maps,
                );
            }

            // A private method defined on `Foo<i32>` should be visible when calling
            // it from an impl on `Foo<i64>`, even though the generics are different.
            if let Some(self_type) = self_type
                && is_same_type_regardless_generics(self_type, object_type)
            {
                if modifiers.visibility.is_private() {
                    // Only allow accessing private methods of a type if we are under the same
                    // module where the type was defined. OTOH if we are in an `impl Foo<i32>`
                    // block in a different module, extending the type with new methods, then
                    // we should only access public parts defined in other modules, or private
                    // ones defined in the same extension.
                    let def_map = &def_maps[&current_module.krate];
                    // Cannot call `type_member_is_visible` because it goes up to the parent;
                    // the `func_meta.source_module` already seems to be the parent.
                    return module_is_descendant_of_target(
                        def_map,
                        current_module.local_id,
                        func_meta.source_module,
                    );
                } else {
                    // If visibility is PublicCrate, then we are good, because is_same_type_regardless_generics
                    // already checked that the types are the same, so we are in the same crate.
                    return true;
                }
            }

            if let Some(struct_id) = func_meta.type_id {
                // For inherent impl methods, check visibility against the impl's
                // defining module (source_module). This prevents private methods defined
                // in `impl super::S` inside `mod private` from being callable outside
                // `mod private` via `s.method()`.
                if func_meta.trait_impl.is_none() {
                    let source_module = ModuleId {
                        krate: func_meta.source_crate,
                        local_id: func_meta.source_module,
                    };
                    let type_module = struct_id.module_id();
                    if source_module != type_module {
                        return item_in_module_is_visible(
                            def_maps,
                            current_module,
                            source_module,
                            modifiers.visibility,
                        );
                    }
                }

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

#[cfg(test)]
mod tests {
    use super::*;

    fn array_type(element: Type, length: u32) -> Type {
        Type::Array(Box::new(element), Box::new(Type::constant_u32(length)))
    }

    #[test]
    fn composite_primitive_types_must_match_exactly_for_visibility() {
        let array_u32_4 = array_type(Type::u32(), 4);
        let array_u32_8 = array_type(Type::u32(), 8);
        let array_bool_4 = array_type(Type::Bool, 4);

        assert!(is_same_type_regardless_generics(&array_u32_4, &array_u32_4));
        assert!(!is_same_type_regardless_generics(&array_u32_4, &array_u32_8));
        assert!(!is_same_type_regardless_generics(&array_u32_4, &array_bool_4));

        let tuple_u32 = Type::Tuple(vec![Type::u32()]);
        let tuple_bool = Type::Tuple(vec![Type::Bool]);
        assert!(!is_same_type_regardless_generics(&tuple_u32, &tuple_bool));

        let function_u32 =
            Type::Function(vec![Type::u32()], Box::new(Type::u32()), Box::new(Type::Unit), false);
        let function_bool =
            Type::Function(vec![Type::Bool], Box::new(Type::u32()), Box::new(Type::Unit), false);
        assert!(!is_same_type_regardless_generics(&function_u32, &function_bool));
    }

    #[test]
    fn references_still_compare_by_inner_type_for_visibility() {
        let array = array_type(Type::u32(), 4);
        let reference = Type::Reference(Box::new(array.clone()), false);

        assert!(is_same_type_regardless_generics(&reference, &array));
        assert!(is_same_type_regardless_generics(&array, &reference));
    }
}
