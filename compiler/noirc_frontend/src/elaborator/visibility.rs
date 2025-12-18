//! Visibility checking for functions, struct fields, and type privacy.

use noirc_errors::{Located, Location};

use crate::{
    DataType, StructField, Type,
    ast::{Ident, ItemVisibility, NoirStruct},
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
    /// Checks whether calling the method `func_id` on an object of type `object_type` is allowed
    /// from the current location. If not, a visibility error is pushed to the error vector.
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
    /// error is pushed to the error vector.
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
        match typ {
            Type::DataType(struct_type, generics) => {
                let struct_type = struct_type.borrow();
                let struct_module_id = struct_type.id.module_id();

                // We only check this in types in the same crate. If it's in a different crate
                // then it's either accessible (all good) or it's not, in which case a different
                // error will happen somewhere else, but no need to error again here.
                if struct_module_id.krate == self.crate_id {
                    let aliased_visibility = self.find_struct_visibility(&struct_type);
                    if aliased_visibility < visibility {
                        self.push_err(ResolverError::TypeIsMorePrivateThenItem {
                            typ: struct_type.name.to_string(),
                            item: name.to_string(),
                            location,
                        });
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
            Type::Reference(typ, _) | Type::Array(_, typ) | Type::Vector(typ) => {
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
