use noirc_errors::Location;

use crate::{
    DataType, Type,
    ast::{Ident, ItemVisibility},
    hir::resolution::{
        errors::ResolverError,
        import::PathResolutionError,
        visibility::{method_call_is_visible, struct_member_is_visible},
    },
    hir_def::function::FuncMeta,
    node_interner::{FuncId, FunctionModifiers},
};

use super::Elaborator;

impl Elaborator<'_> {
    pub(super) fn check_method_call_visibility(
        &mut self,
        func_id: FuncId,
        object_type: &Type,
        name: &Ident,
    ) {
        if !method_call_is_visible(
            self.self_type.as_ref(),
            object_type,
            func_id,
            self.module_id(),
            self.interner,
            self.def_maps,
        ) {
            self.push_err(ResolverError::PathResolutionError(PathResolutionError::Private(
                name.clone(),
            )));
        }
    }

    pub(super) fn check_struct_field_visibility(
        &mut self,
        struct_type: &DataType,
        field_name: &str,
        visibility: ItemVisibility,
        location: Location,
    ) {
        if self.silence_field_visibility_errors > 0 {
            return;
        }

        if !struct_member_is_visible(struct_type.id, visibility, self.module_id(), self.def_maps) {
            self.push_err(ResolverError::PathResolutionError(PathResolutionError::Private(
                Ident::new(field_name.to_string(), location),
            )));
        }
    }

    /// Find the struct in the parent module so we can know its visibility
    pub(super) fn find_struct_visibility(&self, struct_type: &DataType) -> Option<ItemVisibility> {
        let parent_module_id = struct_type.id.parent_module_id(self.def_maps);
        let parent_module_data = self.get_module(parent_module_id);
        let per_ns = parent_module_data.find_name(&struct_type.name);
        per_ns.types.map(|(_, vis, _)| vis)
    }

    /// Check whether a functions return value and args should be checked for private type visibility.
    pub(super) fn should_check_function_visibility(
        &self,
        func_meta: &FuncMeta,
        modifiers: &FunctionModifiers,
    ) -> bool {
        // Private functions don't leak anything.
        if modifiers.visibility == ItemVisibility::Private {
            return false;
        }
        // Implementing public traits on private types is okay, they can't be used unless the type itself is accessible.
        if func_meta.trait_impl.is_some() {
            return false;
        }
        // Public struct functions should not expose private types.
        if let Some(struct_visibility) = func_meta.type_id.and_then(|id| {
            let struct_def = self.get_type(id);
            let struct_def = struct_def.borrow();
            self.find_struct_visibility(&struct_def)
        }) {
            return struct_visibility != ItemVisibility::Private;
        }
        // Standalone functions should be checked
        true
    }

    /// Check that an item such as a struct field or type alias is not more visible than the type it refers to.
    pub(super) fn check_type_is_not_more_private_then_item(
        &mut self,
        name: &Ident,
        visibility: ItemVisibility,
        typ: &Type,
        location: Location,
    ) {
        match typ {
            Type::DataType(struct_type, generics) => {
                let struct_type = struct_type.borrow();
                let struct_module_id = struct_type.id.module_id();

                // We only check this in types in the same crate. If it's in a different crate
                // then it's either accessible (all good) or it's not, in which case a different
                // error will happen somewhere else, but no need to error again here.
                if struct_module_id.krate == self.crate_id {
                    if let Some(aliased_visibility) = self.find_struct_visibility(&struct_type) {
                        if aliased_visibility < visibility {
                            self.push_err(ResolverError::TypeIsMorePrivateThenItem {
                                typ: struct_type.name.to_string(),
                                item: name.to_string(),
                                location,
                            });
                        }
                    }
                }

                for generic in generics {
                    self.check_type_is_not_more_private_then_item(
                        name, visibility, generic, location,
                    );
                }
            }
            Type::Tuple(types) => {
                for typ in types {
                    self.check_type_is_not_more_private_then_item(name, visibility, typ, location);
                }
            }
            Type::Alias(alias_type, generics) => {
                self.check_type_is_not_more_private_then_item(
                    name,
                    visibility,
                    &alias_type.borrow().get_type(generics),
                    location,
                );
            }
            Type::CheckedCast { from, to } => {
                self.check_type_is_not_more_private_then_item(name, visibility, from, location);
                self.check_type_is_not_more_private_then_item(name, visibility, to, location);
            }
            Type::Function(args, return_type, env, _) => {
                for arg in args {
                    self.check_type_is_not_more_private_then_item(name, visibility, arg, location);
                }
                self.check_type_is_not_more_private_then_item(
                    name,
                    visibility,
                    return_type,
                    location,
                );
                self.check_type_is_not_more_private_then_item(name, visibility, env, location);
            }
            Type::Reference(typ, _) | Type::Array(_, typ) | Type::Slice(typ) => {
                self.check_type_is_not_more_private_then_item(name, visibility, typ, location);
            }
            Type::InfixExpr(left, _op, right, _) => {
                self.check_type_is_not_more_private_then_item(name, visibility, left, location);
                self.check_type_is_not_more_private_then_item(name, visibility, right, location);
            }
            Type::FieldElement
            | Type::Integer(..)
            | Type::Bool
            | Type::String(..)
            | Type::FmtString(..)
            | Type::Unit
            | Type::Quoted(..)
            | Type::TypeVariable(..)
            | Type::Forall(..)
            | Type::TraitAsType(..)
            | Type::Constant(..)
            | Type::NamedGeneric(..)
            | Type::Error => (),
        }
    }
}
