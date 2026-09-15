//! The part of an item's context holding the generics and trait bounds that are in scope.

use std::collections::BTreeSet;

use iter_extended::vecmap;

use crate::{
    Type, TypeVariable,
    hir_def::{traits::TraitConstraint, types::ResolvedGeneric},
    node_interner::TraitId,
};

/// The generic parameters an item introduces, together with the trait bounds they carry.
///
/// The bounds live here rather than in a context of their own because a bound is what gives a
/// generic parameter its members: resolving `T::AssociatedType` needs both the parameter and the
/// `where` clause naming the trait that declares the associated type.
///
/// Generics nest - a method's generics are added on top of its impl's - so this is entered and
/// left as a stack via [`Self::enter_scope`] and [`Self::exit_scope`] rather than replaced.
#[derive(Default)]
pub(crate) struct GenericsContext {
    /// Contains a mapping of the current struct or functions's generics to
    /// unique type variables if we're resolving a struct. Empty otherwise.
    /// This is a Vec rather than a map to preserve the order a functions generics
    /// were declared in.
    params: Vec<ResolvedGeneric>,

    /// Each constraint in the `where` clause of the function currently being resolved.
    trait_bounds: Vec<TraitConstraint>,

    /// Every `(object type, trait)` pair brought into scope by implication rather than by a
    /// `where` clause naming it: a bound declared on an associated type, or a parent trait.
    ///
    /// A written bound which duplicates one of these is not reported as unnecessary. The two can
    /// be registered in either order - the implication may come from a later clause in the same
    /// `where` list - so remembering the implied pairs is what makes the diagnostic independent of
    /// that order. Emptied together with the assumed impls by
    /// [`Elaborator::remove_trait_constraints_from_scope`](crate::elaborator::Elaborator::remove_trait_constraints_from_scope).
    implied_trait_bounds: BTreeSet<(Type, TraitId)>,
}

/// Saved generics state for restoration after a scope exits.
pub(crate) struct GenericsState {
    params_count: usize,
}

impl GenericsContext {
    /// A context with `params` in scope, holding the caller's `trait_bounds`.
    pub(crate) fn new(params: Vec<ResolvedGeneric>, trait_bounds: Vec<TraitConstraint>) -> Self {
        Self { params, trait_bounds, implied_trait_bounds: BTreeSet::new() }
    }

    /// Saves the current generics state to be restored later with [`Self::exit_scope`].
    /// Note that all of the generics in scope will still be in scope after this call. This will
    /// only save the position of the current generics so that any generics added afterward can
    /// later be discarded via a call to [`Self::exit_scope`].
    pub(crate) fn enter_scope(&self) -> GenericsState {
        GenericsState { params_count: self.params.len() }
    }

    /// Restores the generics state saved by [`Self::enter_scope`].
    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn exit_scope(&mut self, state: GenericsState) {
        self.params.truncate(state.params_count);
    }

    /// The generics in scope, in the order they were declared in.
    pub(crate) fn params(&self) -> &[ResolvedGeneric] {
        &self.params
    }

    /// Brings one more generic into scope, after the ones already there.
    pub(crate) fn add_param(&mut self, param: ResolvedGeneric) {
        self.params.push(param);
    }

    /// Brings several more generics into scope, after the ones already there.
    pub(crate) fn add_params(&mut self, params: impl IntoIterator<Item = ResolvedGeneric>) {
        self.params.extend(params);
    }

    /// Replaces the generics in scope wholesale, for an item whose generics are known up front
    /// rather than accumulated as its declaration is resolved.
    pub(crate) fn set_params(&mut self, params: Vec<ResolvedGeneric>) {
        self.params = params;
    }

    /// Takes the generics out of scope and hands them to the caller.
    pub(crate) fn take_params(&mut self) -> Vec<ResolvedGeneric> {
        std::mem::take(&mut self.params)
    }

    /// Takes every generic out of scope.
    pub(crate) fn clear_params(&mut self) {
        self.params.clear();
    }

    /// Find a generic variable among the generics of the current struct or function.
    pub(crate) fn find(&self, target_name: &str) -> Option<&ResolvedGeneric> {
        self.params.iter().find(|generic| generic.name.as_ref() == target_name)
    }

    /// The type variables of the generics in scope, in declaration order.
    pub(crate) fn type_vars(&self) -> Vec<TypeVariable> {
        vecmap(&self.params, |generic| generic.type_var.clone())
    }

    /// Whether any bound at all is in scope, written or implied.
    pub(crate) fn has_bounds(&self) -> bool {
        !self.trait_bounds.is_empty()
    }

    /// Assumes `constraint` for the rest of the item.
    pub(crate) fn add_bound(&mut self, constraint: TraitConstraint) {
        self.trait_bounds.push(constraint);
    }

    /// Whether `object_type: trait` is one of the bounds in scope.
    pub(crate) fn has_bound(&self, object_type: &Type, trait_id: TraitId) -> bool {
        self.trait_bounds
            .iter()
            .any(|bound| bound.trait_bound.trait_id == trait_id && bound.typ == *object_type)
    }

    /// The bounds written on the generic parameter named `name`, e.g. both `Foo` and `Bar` for
    /// `T` given `where T: Foo, T: Bar`.
    pub(crate) fn bounds_on_generic<'a>(
        &'a self,
        name: &'a str,
    ) -> impl Iterator<Item = &'a TraitConstraint> {
        self.trait_bounds.iter().filter(move |constraint| {
            matches!(&constraint.typ, Type::NamedGeneric(generic) if generic.name.as_str() == name)
        })
    }

    /// Drops every bound on `trait_id`, for when the constraints that brought them into scope
    /// leave it.
    pub(crate) fn remove_bounds_on_trait(&mut self, trait_id: TraitId) {
        self.trait_bounds.retain(|bound| bound.trait_bound.trait_id != trait_id);
    }

    /// Records `object_type: trait` as a bound that was implied rather than written, so that a
    /// `where` clause repeating it is not reported as unnecessary.
    pub(crate) fn record_implied_bound(&mut self, object_type: &Type, trait_id: TraitId) {
        self.implied_trait_bounds.insert((object_type.clone(), trait_id));
    }

    /// Whether `object_type: trait` was brought into scope by implication.
    pub(crate) fn is_implied_bound(&self, object_type: &Type, trait_id: TraitId) -> bool {
        self.implied_trait_bounds.contains(&(object_type.clone(), trait_id))
    }

    /// Forgets the implied bounds, for when the bounds they were implied by leave scope.
    pub(crate) fn clear_implied_bounds(&mut self) {
        self.implied_trait_bounds.clear();
    }
}
