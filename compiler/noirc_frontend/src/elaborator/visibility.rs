//! Visibility checking for functions, struct fields, and type privacy.

use noirc_errors::{Located, Location};

use crate::{
    DataType, StructField, Type,
    ast::{Ident, ItemVisibility, NoirStruct},
    hir::{
        def_map::ModuleId,
        resolution::{
            errors::ResolverError,
            import::PathResolutionError,
            visibility::{
                item_in_module_is_visible, method_call_is_visible, struct_member_is_visible,
            },
        },
    },
    hir_def::function::FuncMeta,
    node_interner::{FuncId, FunctionModifiers},
};

use super::Elaborator;

/// Describes how to determine whether a referenced type is "too private" for an item.
enum VisibilityCheck {
    /// The referenced type's visibility must be at least this level.
    /// Used for struct fields, type aliases, and function signatures.
    Visibility(ItemVisibility),
    /// The referenced type must be visible from this module.
    /// Used for cross-module impl methods where types are hoisted to a different module.
    VisibleFromModule(ModuleId),
}

impl Elaborator<'_> {
    /// Checks whether calling the method `func_id` on an object of type `object_type` is allowed
    /// from the current location. If not, a visibility error is pushed to the error list.
    /// The passed `name` is used for error reporting.
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

    /// Checks that a public struct does not have fields with more private types.
    ///
    /// For example, a public struct cannot have a public field of a private type,
    /// as this would allow external code to access the private type through the public struct.
    pub(super) fn check_struct_field_type_visibility(
        &mut self,
        struct_def: &NoirStruct,
        fields: &[StructField],
    ) {
        if !struct_def.visibility.is_private() {
            for field in fields {
                let ident = Ident::from(Located::from(
                    field.name.location(),
                    format!("{}::{}", struct_def.name, field.name),
                ));
                self.check_type_is_not_more_private_then_item(
                    &ident,
                    field.visibility,
                    &field.typ,
                    field.name.location(),
                );
            }
        }
    }

    /// Checks whether accessing the struct field `field_name` of type `struct_type`, that has
    /// the given `visibility`, is allowed from the current location. If not, a visibility
    /// error is pushed to the error list.
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

    /// Returns a struct's visibility.
    pub(super) fn find_struct_visibility(&self, struct_type: &DataType) -> ItemVisibility {
        let parent_module_id = struct_type.id.parent_module_id(self.def_maps);
        let parent_module_data = self.get_module(parent_module_id);
        let per_ns = parent_module_data.find_name(&struct_type.name);
        let (_, visibility, _) =
            per_ns.types.expect("Expected to find struct in its parent module");
        visibility
    }

    pub(super) fn check_function_visibility(
        &mut self,
        func_meta: &FuncMeta,
        modifiers: &FunctionModifiers,
        name: &Ident,
        location: Location,
    ) {
        // Check arg and return-value visibility of standalone functions.
        if self.should_check_function_args_and_return_are_not_more_private_than_function(
            func_meta, modifiers,
        ) {
            for (_, typ, _) in func_meta.parameters.iter() {
                self.check_type_is_not_more_private_then_item(
                    name,
                    modifiers.visibility,
                    typ,
                    location,
                );
            }
            self.check_type_is_not_more_private_then_item(
                name,
                modifiers.visibility,
                func_meta.return_type(),
                location,
            );
        }

        // Check that cross-module impl methods don't leak types
        // that are private to the impl's module.
        self.check_cross_module_impl_type_visibility(func_meta, name, location);
    }

    /// Check whether a function's args and return value should be checked for private type visibility.
    pub(super) fn should_check_function_args_and_return_are_not_more_private_than_function(
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
        // Non-private struct functions should not expose private types.
        if let Some(struct_id) = func_meta.type_id {
            let struct_def = self.get_type(struct_id);
            let struct_def = struct_def.borrow();
            let visibility = self.find_struct_visibility(&struct_def);
            return visibility != ItemVisibility::Private;
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
        self.check_type_privacy(name, &VisibilityCheck::Visibility(visibility), typ, location);
    }

    /// For methods defined in cross-module impls (where the impl block is in a different
    /// module than the struct), check that all types in the method's signature are visible
    /// from the struct's defining module.
    fn check_cross_module_impl_type_visibility(
        &mut self,
        func_meta: &FuncMeta,
        name: &Ident,
        location: Location,
    ) {
        let Some(struct_id) = func_meta.type_id else { return };

        if func_meta.trait_impl.is_some() {
            return;
        }

        let struct_parent_module = struct_id.parent_module_id(self.def_maps);
        let impl_module =
            ModuleId { krate: func_meta.source_crate, local_id: func_meta.source_module };

        if struct_parent_module == impl_module {
            return;
        }

        let check = VisibilityCheck::VisibleFromModule(struct_parent_module);
        for (_, typ, _) in func_meta.parameters.iter() {
            self.check_type_privacy(name, &check, typ, location);
        }
        self.check_type_privacy(name, &check, func_meta.return_type(), location);
    }

    /// Recursively walks a type and checks that every DataType referenced in it
    /// satisfies the given visibility check.
    fn check_type_privacy(
        &mut self,
        name: &Ident,
        check: &VisibilityCheck,
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
                let check_crate = match check {
                    VisibilityCheck::Visibility(_) => self.crate_id,
                    VisibilityCheck::VisibleFromModule(module) => module.krate,
                };
                if struct_module_id.krate == check_crate {
                    let aliased_visibility = self.find_struct_visibility(&struct_type);
                    let is_private = match check {
                        VisibilityCheck::Visibility(visibility) => aliased_visibility < *visibility,
                        VisibilityCheck::VisibleFromModule(from_module) => {
                            let target_module = struct_type.id.parent_module_id(self.def_maps);
                            !item_in_module_is_visible(
                                self.def_maps,
                                *from_module,
                                target_module,
                                aliased_visibility,
                            )
                        }
                    };
                    if is_private {
                        self.push_err(ResolverError::TypeIsMorePrivateThenItem {
                            typ: struct_type.name.to_string(),
                            item: name.to_string(),
                            location,
                        });
                    }
                }

                for generic in generics {
                    self.check_type_privacy(name, check, generic, location);
                }
            }
            Type::Tuple(types) => {
                for typ in types {
                    self.check_type_privacy(name, check, typ, location);
                }
            }
            Type::Alias(alias_type, generics) => {
                self.check_type_privacy(
                    name,
                    check,
                    &alias_type.borrow().get_type(generics),
                    location,
                );
            }
            Type::CheckedCast { from, to } => {
                self.check_type_privacy(name, check, from, location);
                self.check_type_privacy(name, check, to, location);
            }
            Type::Function(args, return_type, env, _) => {
                for arg in args {
                    self.check_type_privacy(name, check, arg, location);
                }
                self.check_type_privacy(name, check, return_type, location);
                self.check_type_privacy(name, check, env, location);
            }
            Type::Reference(typ, _) | Type::Array(_, typ) | Type::Vector(typ) => {
                self.check_type_privacy(name, check, typ, location);
            }
            Type::InfixExpr(left, _op, right, _) => {
                self.check_type_privacy(name, check, left, location);
                self.check_type_privacy(name, check, right, location);
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
