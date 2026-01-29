//! Trait definition collection, bounds resolution, and associated types.
//!
//! # Terminology:
//!
//! ## TraitConstraint & TraitBound
//!
//! In the following code:
//! ```noir
//! fn foo<T: Eq>(x: T) -> bool {
//!     x.eq(x)
//! }
//! ```
//! We call `T: Eq` a TraitConstraint, while `Eq` alone (along with any generics)
//! is the TraitBound (although the two are sometimes informally used interchangeably).
//!
//! ## Assumed Implementations
//!
//! A "real" trait implementation corresponds to an `impl` block in noir source code.
//! We can also have "assumed" impls though. These are implementations that we assume
//! to exist, but may not. These most often correspond to trait constraints on generic
//! functions:
//!
//! ```noir
//! fn foo<T: Eq>(x: T) {}
//! ```
//!
//! Locally within `foo`, we say that `T: Eq` is an assumed impl. Within the body of `foo`,
//! we can assume such an impl exists even if there are no impls for `Eq` at all in the program.
//! It is up to the caller to find an impl when the type of `T` becomes known.
//!
//! Assumed impls may be present anywhere a generic trait constraint may be.
//!
//! ## Impl Candidate (or just candidate)
//!
//! An impl candidate is any impl being considered as a potential solution when solving a trait
//! constraint. An impl candidate may be any trait impl for the same trait as the one in the trait
//! constraint, including assumed impls.
//!
//! ## Solving a TraitConstraint
//!
//! Solving a trait constraint is finding the single matching impl candidate it refers to.
//! If it may refer to zero or more than one, the constraint can't be solved and an error should be
//! issued.
//!
//! # Explanation of Core Concepts
//!
//! ## Self
//!
//! In addition to its declared generics, traits have an additional implicit generic
//! called `Self`. This is not stored in the normal list of generics on a trait so it often
//! must be specially handled.
//!
//! When we have a trait and an impl:
//!
//! ```noir
//! trait Foo<A> {
//!     fn foo<B>();
//! }
//!
//! impl Foo<i32> for Bar {
//!     fn foo<B>(){}
//! }
//!
//! fn caller<T, U>() where T: Foo<U> { ... }
//! ```
//!
//! The expected trait to impl bindings would be `[Self => Bar, A => i32]`. `B` in the example
//! above is on the `foo` method itself rather than the trait or impl. If `B` were bound to a
//! concrete type like `u32` in the impl bindings, `foo` would no longer be properly generic.
//!
//! Inlining `Self` into a trait's generics list directly may provide some
//! intuition in how `Self` should be handled (that is, like any other trait generic):
//!
//! ```noir
//! fn caller<T, U>() where Foo<T, U> { ... }
//! ```
//!
//! ## Associated Types & Associated Constants
//!
//! Associated types and associated constants are both represented internally as associated
//! types. Constants are represented as `Type::Constant` variants or `Type::InfixExpr` when
//! operators are involved such as `N + 1`. Generally, this representation is non-leaky and
//! there are very few locations where we need to distinguish between associated types & constants.
//!
//! ```noir
//! trait Foo<A> {
//!     type B;
//!     let C: u32;
//!     fn foo<D>();
//! }
//!
//! impl Foo<i32> for Bar {
//!     type B = Field;
//!     let C: u32 = 42;
//!     fn foo<D>() {}
//! }
//!
//! fn caller<T, U>() where T: Foo<U> {}
//! ```
//!
//! Similar to the implicit `Self` generic, associated types (and constants) are also implicit
//! generics on traits - just generics that are restricted to only have one value for a given set of the
//! trait's other generics. For example, we may think of `Foo` above as being `Foo<Self, A, B, C>`
//! internally, but if we already have an implementation for `Foo<i32, i32, i32, 0>`, it'd be
//! invalid to also have an implementation for `Foo<i32, i32, u32, 1>` - because the last two
//! generics are associated types.
//!
//! That said, these are still represented as generics internally because code using them - such as
//! `caller` - still need to be generic over any possible value for these associated types. With
//! this in mind, we can think of `caller` as being equivalent to:
//!
//! ```noir
//! fn caller<T, U, BB, let CC: u32>() where Foo<T, U, BB, CC> {}
//! ```
//!
//! Where `BB` and `CC` are implicitly added generics to the function. These may also be specified
//! explicitly via `T: Foo<U, B = MyB, C = MyC>` but this isn't very relevant to the inner workings
//! of how the compiler handles associated types.
//!
//! ## How TraitConstraints are resolved
//!
//! This section is an attempt at a primer on how TraitConstraints are resolved by the elaborator.
//!
//! The elaborator starts by seeing parsed code and must:
//! 1. Resolve & type-check code (type_check_variable_with_bindings)
//!   - In doing so, determine if the snippet has a trait constraint which needs to be solved
//!   - Some variables have trait constraints because they refer to a generic function with
//!     one or more trait constraints. Others have trait constraints because they directly refer
//!     to a method from a trait. For this later case, we must set the "select the impl" so that
//!     when the constraint is later solved for, the variable is replaced by a variable referring
//!     to the selected impl's method directly. This replacement is done during monomorphization
//!     but we must set the flag during elaboration.
//! 2. Push each required trait constraint to the function context
//!   - When variables are used they are instantiated by the type system. This means we replace
//!     any generics from their definition type with fresh type variables. This mapping is stored
//!     as the `instantiation_bindings` and is later used by the monomorphizer. This mapping
//!     is applied to the trait constraint as well. So if the original constraint was
//!     `T: Foo<U>` where `T` and `U` are generics, the new constraint may be `_0: Foo<_1>` where
//!     `_0` and `_1` are unbound type variables. Because we don't always push down types, we
//!     may not have the constraints needed to solve what `_0` and `_1` are yet. Therefore, we
//!     push the constraint to the function context to solve after type checking the function
//!     instead.
//! 3. When the function is finished being elaborated, go through and solve any trait constraints
//!    that were pushed to the function context.
//!   - Since the function is done being elaborated, we should have more type constraints now which
//!     should hopefully bind the type variables `_0` and `_1` to concrete types. Our new trait
//!     constraint may look like `A: Foo<i32>`.
//!   - For each pushed trait constraint, solve the constraint by looking through the list of all
//!     trait impls in the program for the relevant trait, along with the list of assumed impls.
//!     A constraint is solved when a matching impl is found, along with a matching impl for any
//!     nested trait constraints that impl may require (e.g. `[T]: Eq` requires `T: Eq`).
//!     A matching impl here is simply one for which all types used in the impl unify with all the types
//!     in the trait constraint.
//!   - Although the core idea is simple, we must carefully handle unification bindings such that
//!     we only keep the ones from the impl(s) which were selected. Impls is plural since an impl
//!     can require more trait constraints which need to be solved recursively. These recursive
//!     impl constraints are obtained from the impl definition but care should be taken to
//!     instantiate them with the original instantiation bindings before checking them so that they
//!     are not bound over. Using the Eq example above, we may have the constraint `[i32]: Eq` at
//!     this step which we may solve for, finding `[T]: Eq`. We instantiate the latter with `T := _0` to
//!     `[_0]: Eq` to see if it unifies with `[i32]`, and it does producing `_0 := i32`. The impl
//!     also requires `T: Eq` though, so now we must instantiate this with the impl instantiation
//!     bindings to get `_0: Eq`, and then apply the previous unification binding to get `i32: Eq`,
//!     which is trivially solved by finding the corresponding impl.
//!   - If a single impl candidate is found, it is used. Otherwise, an error is issued.

use std::{collections::BTreeMap, rc::Rc};

use iter_extended::vecmap;
use noirc_errors::Location;

use crate::{
    Kind, NamedGeneric, ResolvedGeneric, Type, TypeBindings, TypeVariable,
    ast::{
        FunctionDefinition, FunctionKind, GenericTypeArgs, Ident, NoirFunction, Path, TraitBound,
        TraitItem, UnresolvedGeneric, UnresolvedTraitConstraint, UnresolvedType,
        UnresolvedTypeData,
    },
    elaborator::{
        PathResolutionMode, PathResolutionTarget, WildcardDisallowedContext,
        path_resolution::PathResolutionItem, types::WildcardAllowed,
    },
    hir::{
        def_collector::dc_crate::UnresolvedTrait,
        type_check::{TypeCheckError, generics::TraitGenerics},
    },
    hir_def::{
        function::FuncMeta,
        traits::{ResolvedTraitBound, TraitConstraint, TraitFunction},
    },
    node_interner::{DependencyId, FuncId, NodeInterner, ReferenceId, TraitId},
};

use super::{Elaborator, generics::GenericsState};

/// State saved when entering a trait scope, used to restore state on exit.
struct TraitScopeState {
    generics: GenericsState,
    current_trait: Option<TraitId>,
    self_type: Option<Type>,
}

impl Elaborator<'_> {
    /// Sets up the elaborator scope for processing a trait.
    /// Returns state that must be passed to `exit_trait_scope` to restore the previous state.
    fn enter_trait_scope(
        &mut self,
        trait_id: TraitId,
        module_id: crate::hir::def_map::LocalModuleId,
    ) -> TraitScopeState {
        let previous_state = TraitScopeState {
            generics: self.enter_generics_scope(),
            current_trait: self.current_trait,
            self_type: self.self_type.clone(),
        };

        self.local_module = Some(module_id);
        self.current_trait = Some(trait_id);

        let the_trait = self.interner.get_trait(trait_id);
        let self_typevar = the_trait.self_type_typevar.clone();
        self.self_type = Some(Type::TypeVariable(self_typevar));

        previous_state
    }

    /// Restores the elaborator state after processing a trait.
    fn exit_trait_scope(&mut self, state: TraitScopeState) {
        self.exit_generics_scope(state.generics);
        self.current_trait = state.current_trait;
        self.self_type = state.self_type;
    }
}

impl Elaborator<'_> {
    /// For each trait:
    /// 1. Desugar any trait constraints using implicit associated types into the explicit form,
    ///    mentioning all associated types.
    /// 2. Resolves the trait's where clause.
    /// 3. Resolves any bounds on associated types
    /// 4. Resolves the trait's bounds (its listed super traits).
    pub fn collect_traits(&mut self, traits: &mut BTreeMap<TraitId, UnresolvedTrait>) {
        for (trait_id, unresolved_trait) in traits {
            let state = self.enter_trait_scope(*trait_id, unresolved_trait.module_id);

            let resolved_generics = self.interner.get_trait(*trait_id).generics.clone();
            self.add_existing_generics(&unresolved_trait.trait_def.generics, &resolved_generics);

            // Transform any constraints omitting their associated types (e.g. `I: Iterator`)
            // into the explicit form (e.g. `I: Iterator<Item = FreshGeneric>`), returning
            // any fresh generics created in the process (`[FreshGeneric]` here).
            let new_generics =
                self.desugar_trait_constraints(&mut unresolved_trait.trait_def.where_clause);

            let new_generics = vecmap(new_generics, |(generic, _bounds)| {
                // TODO: use `_bounds` variable above
                // See https://github.com/noir-lang/noir/issues/8601
                generic
            });
            self.generics.extend(new_generics);

            let where_clause = self.resolve_trait_constraints_and_add_to_scope(
                &unresolved_trait.trait_def.where_clause,
            );
            self.remove_trait_constraints_from_scope(where_clause.iter());

            let mut associated_type_bounds = rustc_hash::FxHashMap::default();
            for item in &unresolved_trait.trait_def.items {
                if let TraitItem::Type { name, bounds } = &item.item {
                    let resolved_bounds = self.resolve_trait_bounds(bounds);
                    associated_type_bounds.insert(name.to_string(), resolved_bounds);
                }
            }

            // Each associated type in this trait is also an implicit generic
            for associated_type in &self.interner.get_trait(*trait_id).associated_types {
                self.generics.push(associated_type.clone());
            }

            let resolved_trait_bounds =
                self.resolve_trait_bounds(&unresolved_trait.trait_def.bounds);
            for bound in &resolved_trait_bounds {
                self.interner.add_trait_dependency(DependencyId::Trait(bound.trait_id), *trait_id);
            }

            // TODO (https://github.com/noir-lang/noir/issues/10642):
            // combine `where_clause` and `resolved_trait_bounds`
            self.interner.update_trait(*trait_id, |trait_def| {
                trait_def.set_trait_bounds(resolved_trait_bounds);
                trait_def.set_where_clause(where_clause);
                trait_def.set_visibility(unresolved_trait.trait_def.visibility);
                trait_def.set_associated_type_bounds(associated_type_bounds);
                trait_def.set_all_generics(self.generics.clone());
            });

            self.exit_trait_scope(state);
        }
    }

    /// Resolve the methods of each trait in an environment where the trait's generics are in scope.
    ///
    /// This mostly consists of resolving each parameter and any trait constraints. The trait
    /// method bodies are not elaborated.
    pub fn collect_trait_methods(&mut self, traits: &mut BTreeMap<TraitId, UnresolvedTrait>) {
        for (trait_id, unresolved_trait) in traits {
            let state = self.enter_trait_scope(*trait_id, unresolved_trait.module_id);

            self.generics = self.interner.get_trait(*trait_id).all_generics.clone();

            let methods = self.resolve_trait_methods(*trait_id, unresolved_trait);

            self.interner.update_trait(*trait_id, |trait_def| {
                trait_def.set_methods(methods);
            });

            self.exit_trait_scope(state);

            // This check needs to be after the trait's methods are set since
            // the interner may set `interner.ordering_type` based on the result type
            // of the Cmp trait, if this is it.
            if self.crate_id.is_stdlib() {
                self.interner.try_add_infix_operator_trait(*trait_id);
                self.interner.try_add_prefix_operator_trait(*trait_id);
            }
        }
    }

    /// Expands any traits in a where clause to mention all associated types if they were
    /// elided by the user. See [Self::add_missing_named_generics] for more detail.
    ///
    /// Returns all newly created generics to be added to this function/trait/impl.
    pub(super) fn desugar_trait_constraints(
        &mut self,
        where_clause: &mut [UnresolvedTraitConstraint],
    ) -> Vec<(ResolvedGeneric, Vec<ResolvedTraitBound>)> {
        where_clause
            .iter_mut()
            .flat_map(|constraint| {
                self.add_missing_named_generics(&constraint.typ, &mut constraint.trait_bound)
            })
            .collect()
    }

    /// For each associated type that isn't mentioned in a trait bound, this adds
    /// the type as an implicit generic to the where clause and returns the newly
    /// created generics in a vector to add to the function/trait/impl later.
    /// For example, this will turn a function using a trait with 2 associated types:
    ///
    /// `fn foo<T>() where T: Foo { ... }`
    ///
    /// into:
    /// `fn foo<T>() where T: Foo<Bar = A, Baz = B> { ... }`
    ///
    /// with a vector of `<A, B>` returned so that the caller can then modify the function to:
    /// `fn foo<T, A, B>() where T: Foo<Bar = A, Baz = B> { ... }`
    fn add_missing_named_generics(
        &mut self,
        object: &UnresolvedType,
        bound: &mut TraitBound,
    ) -> Vec<(ResolvedGeneric, Vec<ResolvedTraitBound>)> {
        let mut added_generics = Vec::new();
        let trait_path = self.validate_path(bound.trait_path.clone());

        let Ok(PathResolutionItem::Trait(trait_id)) =
            self.resolve_path_or_error(trait_path.clone(), PathResolutionTarget::Type)
        else {
            self.push_err(TypeCheckError::ExpectingOtherError {
                message: "add_missing_named_generics: missing trait".to_string(),
                location: trait_path.location,
            });
            return Vec::new();
        };

        let the_trait = self.get_trait(trait_id);
        let trait_name = the_trait.name.to_string();
        let object_name = object.to_string();
        let associated_type_bounds = the_trait.associated_type_bounds.clone();

        for associated_type in &the_trait.associated_types.clone() {
            if !bound
                .trait_generics
                .named_args
                .iter()
                .any(|(name, _)| name.as_str() == *associated_type.name.as_ref())
            {
                // This generic isn't contained in the bound's named arguments,
                // so add it by creating a fresh type variable.
                let new_generic_id = self.interner.next_type_variable_id();
                let kind = associated_type.type_var.kind();
                let type_var = TypeVariable::unbound(new_generic_id, kind);

                let location = bound.trait_path.location;
                let typ = type_var.clone().into_implicit_named_generic(
                    &associated_type.name,
                    Some((object_name.as_str(), trait_name.as_str())),
                );

                let name = match &typ {
                    Type::NamedGeneric(NamedGeneric { name, .. }) => name.clone(),
                    _ => unreachable!("into_implicit_named_generic returns a NamedGeneric"),
                };

                let typ = self.interner.push_quoted_type(typ);
                let typ = UnresolvedTypeData::Resolved(typ).with_location(location);
                let ident = Ident::new(associated_type.name.as_ref().clone(), location);

                let associated_type_bounds = associated_type_bounds
                    .get(associated_type.name.as_str())
                    .cloned()
                    .unwrap_or_default();

                bound.trait_generics.named_args.push((ident, typ));
                added_generics
                    .push((ResolvedGeneric { name, location, type_var }, associated_type_bounds));
            }
        }

        added_generics
    }

    /// This turns function parameters of the form:
    /// `fn foo(x: impl Bar)`
    ///
    /// into
    /// `fn foo<T0_impl_Bar>(x: T0_impl_Bar) where T0_impl_Bar: Bar`
    /// although the fresh type variable is not named internally.
    pub(super) fn desugar_impl_trait_arg(
        &mut self,
        trait_path: Path,
        trait_generics: GenericTypeArgs,
        generics: &mut Vec<TypeVariable>,
        trait_constraints: &mut Vec<TraitConstraint>,
    ) -> Type {
        let new_generic_id = self.interner.next_type_variable_id();

        let new_generic = TypeVariable::unbound(new_generic_id, Kind::Normal);
        generics.push(new_generic.clone());

        let name = format!("impl {trait_path}");
        let generic_type = new_generic.into_named_generic(&Rc::new(name), None);
        let trait_bound = TraitBound { trait_path, trait_generics };

        if let Some(trait_bound) = self.resolve_trait_bound(&trait_bound) {
            let new_constraint = TraitConstraint { typ: generic_type.clone(), trait_bound };
            trait_constraints.push(new_constraint);
        }

        generic_type
    }

    /// Resolves a slice of trait bounds, filtering out any that fail to resolve.
    fn resolve_trait_bounds(&mut self, bounds: &[TraitBound]) -> Vec<ResolvedTraitBound> {
        bounds.iter().filter_map(|bound| self.resolve_trait_bound(bound)).collect()
    }

    /// Resolves a trait bound, marking the trait as referenced.
    pub(super) fn resolve_trait_bound(&mut self, bound: &TraitBound) -> Option<ResolvedTraitBound> {
        self.resolve_trait_bound_inner(bound, PathResolutionMode::MarkAsReferenced)
    }

    /// Resolves a trait bound, marking the trait as used.
    pub(crate) fn use_trait_bound(&mut self, bound: &TraitBound) -> Option<ResolvedTraitBound> {
        self.resolve_trait_bound_inner(bound, PathResolutionMode::MarkAsUsed)
    }

    /// Resolve the given TraitBound, pushing error(s) if the path or any
    /// types used failed to resolve.
    fn resolve_trait_bound_inner(
        &mut self,
        bound: &TraitBound,
        mode: PathResolutionMode,
    ) -> Option<ResolvedTraitBound> {
        let trait_path = self.validate_path(bound.trait_path.clone());
        let the_trait = self.lookup_trait_or_error(trait_path)?;
        let trait_id = the_trait.id;
        let location = bound.trait_path.location;
        let wildcard_allowed = WildcardAllowed::No(WildcardDisallowedContext::TraitBound);

        let (ordered, named) = self.resolve_type_args_inner(
            bound.trait_generics.clone(),
            trait_id,
            location,
            mode,
            wildcard_allowed,
        );

        let trait_generics = TraitGenerics { ordered, named };
        Some(ResolvedTraitBound { trait_id, trait_generics, location })
    }

    /// Adds the given trait constraints to scope as assumed trait impls.
    ///
    /// Since there is no global/local scope distinction for trait constraints,
    /// care should be taken to manually remove these from scope (via
    /// [Self::remove_trait_constraints_from_scope]) after the desired item finishes resolving.
    pub(super) fn add_trait_constraints_to_scope<'a>(
        &mut self,
        constraints: impl Iterator<Item = &'a TraitConstraint>,
        location: Location,
    ) {
        for constraint in constraints {
            self.add_trait_bound_to_scope(
                location,
                &constraint.typ,
                &constraint.trait_bound,
                constraint.trait_bound.trait_id,
            );
        }

        // Also assume `self` implements the current trait if we are inside a trait definition
        if let Some(trait_id) = self.current_trait {
            let the_trait = self.interner.get_trait(trait_id);
            let constraint = the_trait.as_constraint(the_trait.name.location());
            let self_type =
                self.self_type.clone().expect("Expected a self type if there's a current trait");
            self.add_trait_bound_to_scope(
                location,
                &self_type,
                &constraint.trait_bound,
                constraint.trait_bound.trait_id,
            );
        }
    }

    /// The removing counterpart for [Self::add_trait_constraints_to_scope].
    ///
    /// This will only remove assumed trait impls from scope, but this
    /// is always what is desired since true trait impls are permanent.
    pub(super) fn remove_trait_constraints_from_scope<'a>(
        &mut self,
        constraints: impl Iterator<Item = &'a TraitConstraint>,
    ) {
        for constraint in constraints {
            self.interner
                .remove_assumed_trait_implementations_for_trait(constraint.trait_bound.trait_id);
            // Also remove from trait_bounds
            self.trait_bounds.retain(|c| c.trait_bound.trait_id != constraint.trait_bound.trait_id);
        }

        // Also remove the assumed trait implementation for `self` if this is a trait definition
        if let Some(trait_id) = self.current_trait {
            self.interner.remove_assumed_trait_implementations_for_trait(trait_id);
        }
    }

    /// Resolve the given trait constraints and add them to scope as we go.
    /// This second step is necessary to resolve subsequent constraints such
    /// as `<T as Foo>::Bar: Eq` which may lookup an impl which was assumed
    /// by a previous constraint.
    ///
    /// If these constraints are unwanted afterward they should be manually
    /// removed from the interner.
    pub(super) fn resolve_trait_constraints_and_add_to_scope(
        &mut self,
        where_clause: &[UnresolvedTraitConstraint],
    ) -> Vec<TraitConstraint> {
        where_clause
            .iter()
            .filter_map(|constraint| self.resolve_trait_constraint_and_add_to_scope(constraint))
            .collect()
    }

    /// Resolves a trait constraint and adds it to scope as an assumed impl.
    /// This second step is necessary to resolve subsequent constraints such
    /// as `<T as Foo>::Bar: Eq` which may lookup an impl which was assumed
    /// by a previous constraint.
    fn resolve_trait_constraint_and_add_to_scope(
        &mut self,
        constraint: &UnresolvedTraitConstraint,
    ) -> Option<TraitConstraint> {
        let wildcard_allowed = WildcardAllowed::No(WildcardDisallowedContext::TraitConstraint);
        let typ = self.resolve_type(constraint.typ.clone(), wildcard_allowed);
        let trait_bound = self.resolve_trait_bound(&constraint.trait_bound)?;
        let location = constraint.trait_bound.trait_path.location;

        self.add_trait_bound_to_scope(location, &typ, &trait_bound, trait_bound.trait_id);

        let constraint = TraitConstraint { typ, trait_bound };
        // Also add to trait_bounds so that T::AssocType syntax can be resolved
        self.trait_bounds.push(constraint.clone());
        Some(constraint)
    }

    /// Adds an assumed trait implementation for the given object type and trait bound.
    ///
    /// This also recursively adds assumed implementations for any parent traits.
    /// The `starting_trait_id` parameter is used to detect cycles in the trait hierarchy
    /// and prevent infinite recursion.
    ///
    /// If the trait bound is already satisfied, an `UnneededTraitConstraint` error is pushed.
    pub(super) fn add_trait_bound_to_scope(
        &mut self,
        location: Location,
        object: &Type,
        trait_bound: &ResolvedTraitBound,
        starting_trait_id: TraitId,
    ) {
        let trait_id = trait_bound.trait_id;
        let generics = trait_bound.trait_generics.clone();

        if !self.interner.add_assumed_trait_implementation(object.clone(), trait_id, generics) {
            if let Some(the_trait) = self.interner.try_get_trait(trait_id) {
                let trait_name = the_trait.name.to_string();
                let typ = object.clone();
                self.push_err(TypeCheckError::UnneededTraitConstraint {
                    trait_name,
                    typ,
                    location,
                });
            }
        }

        // Also add assumed implementations for the parent traits, if any
        if let Some(trait_bounds) =
            self.interner.try_get_trait(trait_id).map(|the_trait| the_trait.trait_bounds.clone())
        {
            for parent_trait_bound in trait_bounds {
                // Avoid looping forever in case there are cycles
                if parent_trait_bound.trait_id == starting_trait_id {
                    continue;
                }

                let parent_trait_bound =
                    self.instantiate_parent_trait_bound(trait_bound, &parent_trait_bound);
                self.add_trait_bound_to_scope(
                    location,
                    object,
                    &parent_trait_bound,
                    starting_trait_id,
                );
            }
        }
    }

    /// Resolves a trait's methods, but does not elaborate their bodies.
    /// Sets the FuncMeta for each trait method.
    fn resolve_trait_methods(
        &mut self,
        trait_id: TraitId,
        unresolved_trait: &UnresolvedTrait,
    ) -> Vec<TraitFunction> {
        self.local_module = Some(unresolved_trait.module_id);

        let mut functions = vec![];

        for item in &unresolved_trait.trait_def.items {
            if let TraitItem::Function {
                name,
                generics,
                parameters,
                return_type,
                where_clause,
                body,
                is_unconstrained,
                visibility: _,
                is_comptime: _,
            } = &item.item
            {
                self.recover_generics(|this| {
                    let the_trait = this.interner.get_trait(trait_id);
                    let self_typevar = the_trait.self_type_typevar.clone();
                    let name_location = the_trait.name.location();

                    this.add_existing_generic(
                        &UnresolvedGeneric::from(Ident::from("Self")),
                        name_location,
                        &ResolvedGeneric {
                            name: Rc::new("Self".to_owned()),
                            type_var: self_typevar,
                            location: name_location,
                        },
                    );

                    let func_id = unresolved_trait.method_ids[name.as_str()];
                    let mut where_clause = where_clause.to_vec();

                    // Attach any trait constraints on the trait to the function,
                    where_clause.extend(unresolved_trait.trait_def.where_clause.clone());
                    let mut def = FunctionDefinition::normal(
                        name,
                        *is_unconstrained,
                        generics,
                        parameters,
                        body.clone().unwrap_or_default(),
                        where_clause,
                        return_type,
                    );
                    // Trait functions always have the same visibility as the trait they are in
                    def.visibility = unresolved_trait.trait_def.visibility;

                    this.resolve_trait_function(trait_id, func_id, def, body.is_some());

                    if !item.doc_comments.is_empty() {
                        let id = ReferenceId::Function(func_id);
                        this.interner.set_doc_comments(id, item.doc_comments.clone());
                    }

                    let func_meta = this.interner.function_meta(&func_id);

                    let arguments = vecmap(&func_meta.parameters.0, |(_, typ, _)| typ.clone());
                    let return_type = func_meta.return_type().clone();

                    let generics = vecmap(&this.generics, |generic| generic.type_var.clone());

                    let default_impl = unresolved_trait
                        .fns_with_default_impl
                        .functions
                        .iter()
                        .filter(|(_, _, q)| q.name() == name.as_str())
                        .take(2)
                        .fold(None, |opt, item| opt.xor(Some(item)))
                        .map(|(_, _, q)| Box::new(q.clone()));

                    let no_environment = Box::new(Type::Unit);
                    let function_type = Type::Function(
                        arguments,
                        Box::new(return_type),
                        no_environment,
                        *is_unconstrained,
                    );

                    functions.push(TraitFunction {
                        name: name.clone(),
                        typ: Type::Forall(generics, Box::new(function_type)),
                        location: Location::new(name.span(), unresolved_trait.file_id),
                        default_impl,
                        default_impl_module_id: unresolved_trait.module_id,
                        trait_constraints: func_meta.trait_constraints.clone(),
                        direct_generics: func_meta.direct_generics.clone(),
                    });
                });
            }
        }
        functions
    }

    /// Defines the FuncMeta for this trait function.
    ///
    /// The bodies of each function (if they exist) are not elaborated.
    fn resolve_trait_function(
        &mut self,
        trait_id: TraitId,
        func_id: FuncId,
        def: FunctionDefinition,
        has_body: bool,
    ) {
        let old_generic_count = self.generics.len();

        self.scopes.start_function();

        let kind =
            if has_body { FunctionKind::Normal } else { FunctionKind::TraitFunctionWithoutBody };
        let mut function = NoirFunction { kind, def };
        let no_extra_trait_constraints = &[];
        self.define_function_meta(
            &mut function,
            func_id,
            Some(trait_id),
            no_extra_trait_constraints,
        );

        // Here we elaborate functions without a body, mainly to check the arguments and return types.
        // Later on we'll elaborate functions with a body by fully type-checking them.
        if !has_body {
            self.elaborate_function(func_id);
        }

        let _ = self.scopes.end_function();
        // Don't check the scope tree for unused variables, they can't be used in a declaration anyway.
        self.generics.truncate(old_generic_count);
    }
}

/// Checks that the type of a function in a trait impl matches the type
/// of the corresponding function declaration in the trait itself.
///
/// To do this, given a trait such as:
/// `trait Foo<A> { fn foo<B>(...); }`
///
/// And an impl such as:
/// `impl<C> Foo<D> for Bar<E> { fn foo<F>(...); } `
///
/// We have to substitute:
/// - `Self` for `Bar<E>`
/// - `A` for `D`
/// - `B` for `F`
///
/// Before we can type check. Finally, we must also check that the unification
/// result does not introduce any new bindings. This can happen if the impl
/// function's type is more general than that of the trait function. E.g.
/// `fn baz<A, B>(a: A, b: B)` when the impl required `fn baz<A>(a: A, b: A)`.
///
/// This does not type check the body of the impl function.
pub(crate) fn check_trait_impl_method_matches_declaration(
    interner: &NodeInterner,
    function: FuncId,
    noir_function: &NoirFunction,
) -> Vec<TypeCheckError> {
    let meta = interner.function_meta(&function);
    let method_name = interner.function_name(&function);
    let mut errors = Vec::new();

    let impl_id =
        meta.trait_impl.expect("Trait impl function should have a corresponding trait impl");

    // If the trait implementation is not defined in the interner then there was a previous
    // error in resolving the trait path and there is likely no trait for this impl.
    let Some(impl_) = interner.try_get_trait_implementation(impl_id) else {
        errors.push(TypeCheckError::ExpectingOtherError {
            message: "check_trait_impl_method_matches_declaration: missing trait impl".to_string(),
            location: noir_function.def.location,
        });
        return errors;
    };

    let impl_ = impl_.borrow();
    let trait_info = interner.get_trait(impl_.trait_id);

    if trait_info.generics.len() != impl_.trait_generics.len() {
        let expected = trait_info.generics.len();
        let found = impl_.trait_generics.len();
        let location = impl_.ident.location();
        let item = trait_info.name.to_string();
        errors.push(TypeCheckError::GenericCountMismatch { item, expected, found, location });
    }

    // Substitute each generic on the trait with the corresponding generic on the impl
    let mut bindings = interner.trait_to_impl_bindings(
        impl_.trait_id,
        impl_id,
        &impl_.trait_generics,
        impl_.typ.clone(),
    );

    // If this is None, the trait does not have the corresponding function.
    // This error should have been caught in name resolution already so we don't
    // issue an error for it here.
    if let Some(trait_fn_id) = trait_info.method_ids.get(method_name) {
        let trait_fn_meta = interner.function_meta(trait_fn_id);

        if trait_fn_meta.direct_generics.len() != meta.direct_generics.len() {
            let expected = trait_fn_meta.direct_generics.len();
            let found = meta.direct_generics.len();
            let location = meta.name.location;
            let item = method_name.to_string();
            errors.push(TypeCheckError::GenericCountMismatch { item, expected, found, location });
        }

        // Substitute each generic on the trait function with the corresponding generic on the impl function
        for (
            ResolvedGeneric { type_var: trait_fn_generic, .. },
            ResolvedGeneric { name, type_var: impl_fn_generic, .. },
        ) in trait_fn_meta.direct_generics.iter().zip(&meta.direct_generics)
        {
            let trait_fn_kind = trait_fn_generic.kind();
            let arg = impl_fn_generic.clone().into_named_generic(name, None);
            bindings.insert(trait_fn_generic.id(), (trait_fn_generic.clone(), trait_fn_kind, arg));
        }

        let (declaration_type, _) = trait_fn_meta.typ.instantiate_with_bindings(bindings, interner);

        check_function_type_matches_expected_type(
            &declaration_type,
            meta,
            method_name,
            noir_function,
            trait_info.name.as_str(),
            &mut errors,
        );
    } else {
        errors.push(TypeCheckError::ExpectingOtherError {
            message: "check_trait_impl_method_matches_declaration: missing trait method function"
                .to_string(),
            location: meta.name.location,
        });
    }

    errors
}

/// Check the given function type matches the expected one.
///
/// This is used to check if a trait impl's function type matches the declared function in the
/// original trait declaration - while handling the appropriate generic substitutions.
fn check_function_type_matches_expected_type(
    expected: &Type,
    meta: &FuncMeta,
    method_name: &str,
    noir_function: &NoirFunction,
    trait_name: &str,
    errors: &mut Vec<TypeCheckError>,
) {
    let mut bindings = TypeBindings::default();
    let actual = meta.typ.as_monotype();
    let location = meta.name.location;
    if let (
        Type::Function(params_a, ret_a, env_a, unconstrained_a),
        Type::Function(params_b, ret_b, env_b, unconstrained_b),
    ) = (expected, actual)
    {
        // Shouldn't need to unify envs, they should always be equal since they're both free functions
        assert_eq!(env_a, env_b, "envs should match as they're both free functions");

        if unconstrained_a != unconstrained_b {
            errors.push(TypeCheckError::UnconstrainedMismatch {
                item: method_name.to_string(),
                expected: *unconstrained_a,
                location,
            });
        }

        if params_a.len() == params_b.len() {
            for (i, (a, b)) in params_a.iter().zip(params_b.iter()).enumerate() {
                if a.try_unify(b, &mut bindings).is_err() {
                    let parameter_location = noir_function.def.parameters.get(i);
                    let parameter_location = parameter_location.map(|param| param.typ.location);
                    let parameter_location =
                        parameter_location.unwrap_or_else(|| meta.parameters.0[i].0.location());

                    errors.push(TypeCheckError::TraitMethodParameterTypeMismatch {
                        method_name: method_name.to_string(),
                        expected_typ: a.to_string(),
                        actual_typ: b.to_string(),
                        parameter_location,
                        parameter_index: i + 1,
                    });
                }
            }

            if ret_b.try_unify(ret_a, &mut bindings).is_err() {
                errors.push(TypeCheckError::TypeMismatch {
                    expected_typ: ret_a.to_string(),
                    expr_typ: ret_b.to_string(),
                    expr_location: meta.return_type.location(),
                });
            }
        } else {
            errors.push(TypeCheckError::MismatchTraitImplNumParameters {
                actual_num_parameters: params_b.len(),
                expected_num_parameters: params_a.len(),
                trait_name: trait_name.to_string(),
                method_name: method_name.to_string(),
                location,
            });
        }
    }

    // If result bindings is not empty, a type variable was bound which means the two
    // signatures were not a perfect match. Note that this relies on us already binding
    // all the expected generics to each other prior to this check.
    if !bindings.is_empty() {
        let expected_typ = expected.to_string();
        let expr_typ = actual.to_string();
        errors.push(TypeCheckError::TypeMismatch {
            expected_typ,
            expr_typ,
            expr_location: location,
        });
    }
}
