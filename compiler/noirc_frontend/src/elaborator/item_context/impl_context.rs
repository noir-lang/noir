//! The part of an item's context that says which impl or trait the item belongs to.

use crate::{
    Type,
    node_interner::{ImplId, TraitId, TraitImplId},
};

/// Where the item being elaborated sits with respect to impls and traits: what `Self` names, and
/// which trait, trait impl or inherent impl block encloses it.
///
/// The fields are read together - `Self` is meaningless without knowing which kind of block it
/// came from - so they are set together, as a unit, by every scope that enters an impl or a trait.
/// A scope that enters none of them leaves this [`Default`], which is what
/// [`Self::is_outside_any_impl_or_trait`] reports.
#[derive(Default, Clone)]
pub(crate) struct ImplContext {
    /// Set to the current type if we're resolving an impl
    pub(crate) self_type: Option<Type>,

    /// The trait we're currently resolving or implementing, if any.
    /// Set during both trait definitions (`trait Foo { ... }`) and
    /// trait impl elaboration (`impl Foo for Bar { ... }`).
    pub(crate) current_trait: Option<TraitId>,

    /// If we're currently resolving methods within a trait impl, this will be set
    /// to the corresponding trait impl ID.
    pub(crate) current_trait_impl: Option<TraitImplId>,

    /// If we're currently resolving methods within an inherent (non-trait) impl,
    /// this will be set to the corresponding impl ID.
    pub(crate) current_impl: Option<ImplId>,
}

impl ImplContext {
    /// True when no impl or trait encloses the item being elaborated.
    pub(crate) fn is_outside_any_impl_or_trait(&self) -> bool {
        // Destructured rather than accessed field by field so that a field added to this struct
        // has to be considered here.
        let Self { self_type, current_trait, current_trait_impl, current_impl } = self;
        self_type.is_none()
            && current_trait.is_none()
            && current_trait_impl.is_none()
            && current_impl.is_none()
    }

    /// The type `Self` refers to, when the item is inside a trait or a trait impl.
    ///
    /// `None` for an inherent impl: a constraint is only resolved against `Self` when `Self` is
    /// the trait's own type variable or the type a trait impl is written for.
    pub(crate) fn trait_self_type(&self) -> Option<Type> {
        self.current_trait.and_then(|_| self.self_type.clone())
    }
}
