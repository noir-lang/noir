//! Inherent type implementations collection and method declaration.
//!
//! This module handles the collection phase of impl blocks, where methods are declared
//! and registered in their appropriate modules. This is distinct from elaboration, where
//! method bodies are resolved and type-checked (which happens later through regular function
//! elaboration).
//!
//! ## Design
//!
//! The impl collection process occurs in several phases:
//!
//! 1. Function metadata definition
//!    - Resolves the impl's self type (e.g., `impl Foo` → resolves `Foo` to a concrete type)
//!    - Collects function signatures for all methods in the impl block
//!    - This happens early so method signatures are available before bodies are elaborated
//!
//! 2. Method declaration
//!    - Declares methods in the struct's module (not the impl's module)
//!    - Handles method shadowing (inherent impls take precedence over trait impls)
//!    - Validates that impls are only defined for types in the current crate
//!    - This phase makes methods discoverable via `TypeName::method` syntax
//!
//! 3. Method elaboration
//!    - Elaborates method bodies using regular function elaboration
//!    - Type-checks method implementations
//!    - This is straightforward since methods are just functions with an implicit self parameter
//!
//! ## Cross Module Resolution Strategy
//!
//! Impl methods can be declared in one module but resolved in another.
//!
//! - Declaration module: The struct's defining module (where the type was defined)
//!   - Methods are added here so `TypeName::method` resolves correctly
//!   - This allows qualified method calls from anywhere with the right imports
//! - Resolution module: The impl block's module (where the impl appears)
//!   - Names inside method bodies resolve in this scope
//!   - This determines what imports, types, and functions are visible to the method
//!
//! ### Example:
//! ```noir
//! // In module `types`:
//! struct Point { x: Field, y: Field }
//!
//! // In module `methods`:
//! use crate::types::Point;
//! impl Point {
//!     fn distance(self) -> Field {
//!         // This resolves names in `methods` module
//!         helper_function() // looks for helper_function in `methods`
//!     }
//! }
//! ```
//!
//! - `distance` is declared in the `types` module (with `Point`)
//! - `distance` body resolves names in the `methods` module
//! - Users can call `Point::distance` if they import `Point`
//!
//! ## Method Shadowing and Specialization
//!
//! When multiple methods with the same name are declared:
//!
//! 1. Inherent impls shadow trait impls: If both an inherent impl and a trait impl
//!    define the same method, the inherent impl version takes precedence for qualified
//!    calls like `Foo::method()`. The trait impl version is removed from the module scope
//!    to prevent ambiguity.
//! 2. Specialization: Multiple trait impls can define the same method if their
//!    self types don't overlap. The method is removed from module scope when a duplicate
//!    is found, preventing qualified access. However, the methods are still registered
//!    in the interner for dynamic dispatch. Overlap checking happens later during
//!    trait resolution.
//!
//! ## Restrictions
//!
//! - Foreign impl check: Inherent impls are only allowed on types defined in the
//!   current crate. This prevents external crates from adding methods to your types.
//! - Primitive type impls: Only the standard library can impl methods on primitive
//!   types like `Field`, `bool`, `u32`, etc. User code cannot add methods to primitives.

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
    /// Collects all impl blocks for a given type, declaring their methods in the type's module.
    ///
    /// This is called during the impl collection phase (after function metadata has been defined).
    /// It validates impl generics and declares each method so they can be resolved via
    /// `TypeName::method` syntax.
    ///
    /// # Parameters
    /// - `module`: The module where the impl block appears (used for name resolution in method bodies)
    /// - `impls`: All impl blocks for this self type
    /// - `self_type`: The type being implemented (e.g., `Foo` in `impl Foo { ... }`)
    ///
    /// # Panics
    /// If the self_type is not already resolved in each impl's function set.
    /// The self type should be resolved by [Self::define_function_metas] before this method is called.
    pub(super) fn collect_impls(
        &mut self,
        module: LocalModuleId,
        impls: &mut [(UnresolvedGenerics, Location, UnresolvedFunctions)],
        self_type: &UnresolvedType,
    ) {
        self.local_module = Some(module);

        for (generics, location, unresolved) in impls {
            self.check_generics_appear_in_types(generics, &[self_type], &[]);

            self.recover_generics(|this| {
                this.declare_methods_on_struct(None, unresolved, *location);
            });
        }
    }

    /// Declares methods in the appropriate module and registers them in the interner.
    ///
    /// This handles the cross-module strategy: methods are declared in the struct's module
    /// (for qualified calls) but will be resolved in the impl's module (for name resolution).
    ///
    /// # Parameters
    /// - `trait_id`: `Some(trait_id)` if this is a trait impl, `None` for inherent impls
    /// - `functions`: The functions/methods to declare (self_type must already be resolved)
    /// - `location`: Location of the impl block for error reporting
    ///
    /// # Error Cases
    /// - Foreign impl: Attempting to impl a type from another crate
    /// - Duplicate methods: Multiple methods with the same name (handled via shadowing)
    /// - Primitive impl: Non-stdlib code trying to impl methods on primitive types
    ///
    /// # Panics
    /// If the self_type is not already resolved in each impl's function set.
    /// The self type should be resolved by [Self::define_function_metas] before this method is called.
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

        if let Type::DataType(struct_type, _) = &self_type.follow_bindings() {
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

            // Declare each method in the struct's module for qualified access (TypeName::method)
            for (_, method_id, method) in &functions.functions {
                let name = method.name_ident().clone();
                let result = if let Some(trait_id) = trait_id {
                    module.declare_trait_function(name, *method_id, trait_id)
                } else {
                    module.declare_function(name, method.def.visibility, *method_id)
                };

                // Handle method shadowing when a duplicate method name is found
                if result.is_err() {
                    let existing = module.find_func_with_name(method.name_ident()).expect(
                        "declare_function should only error if there is an existing function",
                    );

                    // Inherent impls take precedence over trait impls for qualified calls.
                    // If the existing method is from a trait impl, remove it from module scope
                    // so that `TypeName::method` resolves to the inherent impl version.
                    //
                    // For trait-impl vs trait-impl duplicates, we also remove the existing
                    // method to prevent qualified access. This allows specialization (e.g.,
                    // `impl Trait<A> for Foo` and `impl Trait<B> for Foo` can coexist).
                    // Checking whether the object types in each method overlap (which will be rejected)
                    // happens later during trait resolution.
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
                let is_primitive = self_type.is_primitive();
                self.push_err(DefCollectorErrorKind::NonEnumNonStructTypeInImpl {
                    location,
                    is_primitive,
                });
            }
        }
    }

    /// Registers methods in the interner's method table for dynamic dispatch.
    ///
    /// This associates each method with its self type, enabling method call resolution.
    /// Unlike module declaration (which enables `TypeName::method` syntax), this registration
    /// is required for method calls via the dot syntax (e.g., `value.method()`).
    ///
    /// Trait impl methods are registered separately in `NodeInterner::add_trait_implementation`,
    /// so this is only called for inherent impls.
    ///
    /// # Returns
    /// Errors if a method with the same name is already registered for this type in the interner.
    /// This indicates a true duplicate (not specialization), such as:
    /// ```noir
    /// impl Foo { fn bar(self) {} }
    /// impl Foo { fn bar(self) {} }  // Error: duplicate definition
    /// ```
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
