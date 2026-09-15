//! The part of an item's context that says which impl or trait the item belongs to.

use crate::{
    Type,
    hir_def::function::FuncMeta,
    node_interner::{ImplId, TraitId, TraitImplId, TypeId},
};

/// Where the item being elaborated sits with respect to impls and traits: what `Self` names, and
/// which trait, trait impl or inherent impl block encloses it.
///
/// These fields describe one fact between them - `Self` means nothing without knowing whether it
/// came from a trait, a trait impl or an inherent impl - so the constructors below fill them in as
/// a unit and nothing else can change them afterwards, save for the `Self` type of a comptime
/// function set ([`Self::set_self_type`]). A scope inside no impl or trait leaves this
/// [`Default`], which is what [`Self::is_outside_any_impl_or_trait`] reports.
#[derive(Default, Clone)]
pub(crate) struct ImplContext {
    /// Set to the current type if we're resolving an impl
    self_type: Option<Type>,

    /// The trait we're currently resolving or implementing, if any.
    /// Set during both trait definitions (`trait Foo { ... }`) and
    /// trait impl elaboration (`impl Foo for Bar { ... }`).
    current_trait: Option<TraitId>,

    /// If we're currently resolving methods within a trait impl, this will be set
    /// to the corresponding trait impl ID.
    current_trait_impl: Option<TraitImplId>,

    /// If we're currently resolving methods within an inherent (non-trait) impl,
    /// this will be set to the corresponding impl ID.
    current_impl: Option<ImplId>,
}

impl ImplContext {
    /// Inside `trait Foo { ... }`, where `Self` is the trait's own type variable.
    pub(crate) fn in_trait(current_trait: TraitId, self_type: Type) -> Self {
        Self {
            self_type: Some(self_type),
            current_trait: Some(current_trait),
            current_trait_impl: None,
            current_impl: None,
        }
    }

    /// Inside `impl Foo for Bar { ... }`.
    ///
    /// Each argument is optional because the impl is entered before it is fully resolved: the
    /// trait path may not have named a trait, and the self type is resolved after the header.
    pub(crate) fn in_trait_impl(
        self_type: Option<Type>,
        current_trait: Option<TraitId>,
        current_trait_impl: Option<TraitImplId>,
    ) -> Self {
        Self { self_type, current_trait, current_trait_impl, current_impl: None }
    }

    /// Inside an inherent `impl Bar { ... }`.
    pub(crate) fn in_inherent_impl(current_impl: ImplId, self_type: Type) -> Self {
        Self {
            self_type: Some(self_type),
            current_trait: None,
            current_trait_impl: None,
            current_impl: Some(current_impl),
        }
    }

    /// The impl or trait a function was declared in, as recorded on its [`FuncMeta`].
    pub(crate) fn of_function(meta: &FuncMeta) -> Self {
        Self {
            self_type: meta.self_type.clone(),
            current_trait: meta.trait_id,
            current_trait_impl: meta.trait_impl,
            current_impl: meta.impl_id,
        }
    }

    pub(crate) fn self_type(&self) -> Option<&Type> {
        self.self_type.as_ref()
    }

    pub(crate) fn current_trait(&self) -> Option<TraitId> {
        self.current_trait
    }

    pub(crate) fn current_trait_impl(&self) -> Option<TraitImplId> {
        self.current_trait_impl
    }

    pub(crate) fn current_impl(&self) -> Option<ImplId> {
        self.current_impl
    }

    /// Takes the `Self` type out of scope and hands it to the caller, to be put back with
    /// [`Self::set_self_type`].
    ///
    /// The `Self` type is the one part of this context that a scope narrower than an item can
    /// change: a comptime function set is interpreted with its own `Self` in an enclosing item's
    /// context.
    #[must_use]
    pub(crate) fn take_self_type(&mut self) -> Option<Type> {
        self.self_type.take()
    }

    /// Installs `self_type` as what `Self` names for the rest of the item.
    pub(crate) fn set_self_type(&mut self, self_type: Option<Type>) {
        self.self_type = self_type;
    }

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
