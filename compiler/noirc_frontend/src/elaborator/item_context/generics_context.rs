//! The part of an item's context holding the generics and trait bounds that are in scope.

use std::collections::BTreeSet;

use crate::{
    Type,
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
    pub(crate) params: Vec<ResolvedGeneric>,

    /// Each constraint in the `where` clause of the function currently being resolved.
    pub(crate) trait_bounds: Vec<TraitConstraint>,

    /// Every `(object type, trait)` pair brought into scope by implication rather than by a
    /// `where` clause naming it: a bound declared on an associated type, or a parent trait.
    ///
    /// A written bound which duplicates one of these is not reported as unnecessary. The two can
    /// be registered in either order - the implication may come from a later clause in the same
    /// `where` list - so remembering the implied pairs is what makes the diagnostic independent of
    /// that order. Emptied together with the assumed impls by
    /// [`Elaborator::remove_trait_constraints_from_scope`](crate::elaborator::Elaborator::remove_trait_constraints_from_scope).
    pub(crate) implied_trait_bounds: BTreeSet<(Type, TraitId)>,
}

/// Saved generics state for restoration after a scope exits.
pub(crate) struct GenericsState {
    params_count: usize,
}

impl GenericsContext {
    /// Saves the current generics state to be restored later with [`Self::exit_scope`].
    /// Note that all of `self.params` will still be in scope after this call. This will only save
    /// the position of the current generics so that any generics added afterward can later be
    /// discarded via a call to [`Self::exit_scope`].
    pub(crate) fn enter_scope(&self) -> GenericsState {
        GenericsState { params_count: self.params.len() }
    }

    /// Restores the generics state saved by [`Self::enter_scope`].
    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn exit_scope(&mut self, state: GenericsState) {
        self.params.truncate(state.params_count);
    }

    /// Find a generic variable among the generics of the current struct or function.
    pub(crate) fn find(&self, target_name: &str) -> Option<&ResolvedGeneric> {
        self.params.iter().find(|generic| generic.name.as_ref() == target_name)
    }
}
