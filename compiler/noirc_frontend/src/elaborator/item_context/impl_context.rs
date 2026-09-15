//! The part of an item's context that says which impl or trait the item belongs to.

use crate::{
    Type,
    node_interner::{ImplId, TraitId, TraitImplId, TypeId},
};

/// Where the item being elaborated sits with respect to impls and traits: what `Self` names, and
/// which trait, trait impl or inherent impl block encloses it.
///
/// These fields describe one fact between them - `Self` means nothing without knowing whether it
/// came from a trait, a trait impl or an inherent impl - so a scope entering any of those fills
/// them in as a unit. A scope inside none of them leaves this [`Default`], which is what
/// [`Self::is_outside_any_impl_or_trait`] reports.
#[derive(Default)]
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

    /// The struct or enum `Self` names, when `Self` is one.
    ///
    /// `None` when there is no `Self` in scope and when `Self` is something other than a data
    /// type, such as the trait's own type variable inside a trait definition.
    pub(crate) fn self_data_type_id(&self) -> Option<TypeId> {
        match self.self_type.as_ref()? {
            Type::DataType(data_type, _) => Some(data_type.borrow().id),
            _ => None,
        }
    }

    /// The type `Self` refers to, when the item is inside a trait or a trait impl.
    ///
    /// `None` for an inherent impl: a constraint is only resolved against `Self` when `Self` is
    /// the trait's own type variable or the type a trait impl is written for.
    pub(crate) fn trait_self_type(&self) -> Option<Type> {
        self.current_trait?;
        self.self_type.clone()
    }
}
