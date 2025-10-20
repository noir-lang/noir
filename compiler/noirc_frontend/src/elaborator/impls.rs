//! Inherent type implementations collection and method declaration.

use noirc_errors::Location;

use crate::{
    Type,
    ast::{UnresolvedGenerics, UnresolvedType},
    hir::{
        def_collector::{dc_crate::UnresolvedFunctions, errors::DefCollectorErrorKind},
        def_map::LocalModuleId,
        resolution::errors::ResolverError,
    },
    node_interner::{FuncId, TraitId},
};

use super::Elaborator;

impl Elaborator<'_> {
    pub(super) fn collect_impls(
        &mut self,
        module: LocalModuleId,
        impls: &mut [(UnresolvedGenerics, Location, UnresolvedFunctions)],
        self_type: &UnresolvedType,
    ) {
        self.local_module = module;

        for (generics, location, unresolved) in impls {
            self.check_generics_appear_in_types(generics, &[self_type], &[]);

            let old_generic_count = self.generics.len();
            self.add_generics(generics);
            self.declare_methods_on_struct(None, unresolved, *location);
            self.generics.truncate(old_generic_count);
        }
    }

    pub(super) fn declare_methods_on_struct(
        &mut self,
        trait_id: Option<TraitId>,
        functions: &mut UnresolvedFunctions,
        location: Location,
    ) {
        let self_type = functions.self_type.as_ref();
        let self_type =
            self_type.expect("Expected struct type to be set before declare_methods_on_struct");

        let function_ids = functions.function_ids();

        if let Type::DataType(struct_type, _) = &self_type {
            let struct_ref = struct_type.borrow();

            // `impl`s are only allowed on types defined within the current crate
            if trait_id.is_none() && struct_ref.id.krate() != self.crate_id {
                let type_name = struct_ref.name.to_string();
                self.push_err(DefCollectorErrorKind::ForeignImpl { location, type_name });
                return;
            }

            // Grab the module defined by the struct type. Note that impls are a case
            // where the module the methods are added to is not the same as the module
            // they are resolved in.
            let module = Self::get_module_mut(self.def_maps, struct_ref.id.module_id());

            for (_, method_id, method) in &functions.functions {
                // If this method was already declared, remove it from the module so it cannot
                // be accessed with the `TypeName::method` syntax. We'll check later whether the
                // object types in each method overlap or not. If they do, we issue an error.
                // If not, that is specialization which is allowed.
                let name = method.name_ident().clone();
                let result = if let Some(trait_id) = trait_id {
                    module.declare_trait_function(name, *method_id, trait_id)
                } else {
                    module.declare_function(name, method.def.visibility, *method_id)
                };
                if result.is_err() {
                    let existing = module.find_func_with_name(method.name_ident()).expect(
                        "declare_function should only error if there is an existing function",
                    );

                    // Only remove the existing function from scope if it is from a trait impl as
                    // well. If it is from a non-trait impl that should override trait impl methods
                    // anyway so that Foo::bar always resolves to the non-trait impl version.
                    if self.interner.function_meta(&existing).trait_impl.is_some() {
                        module.remove_function(method.name_ident());
                    }
                }
            }

            // Trait impl methods are already declared in NodeInterner::add_trait_implementation
            if trait_id.is_none() {
                self.declare_methods(self_type, &function_ids);
            }
        // We can define methods on primitive types only if we're in the stdlib
        } else if trait_id.is_none() && *self_type != Type::Error {
            if self.crate_id.is_stdlib() {
                // Trait impl methods are already declared in NodeInterner::add_trait_implementation
                if trait_id.is_none() {
                    self.declare_methods(self_type, &function_ids);
                }
            } else {
                self.push_err(DefCollectorErrorKind::NonStructTypeInImpl { location });
            }
        }
    }

    fn declare_methods(&mut self, self_type: &Type, function_ids: &[FuncId]) {
        for method_id in function_ids {
            let method_name = self.interner.function_name(method_id).to_owned();

            if let Some(first_fn) =
                self.interner.add_method(self_type, method_name.clone(), *method_id, None)
            {
                let first_location = self.interner.function_ident(&first_fn).location();
                let second_location = self.interner.function_ident(method_id).location();
                let error = ResolverError::DuplicateDefinition {
                    name: method_name,
                    first_location,
                    second_location,
                };
                self.push_err(error);
            }
        }
    }
}
