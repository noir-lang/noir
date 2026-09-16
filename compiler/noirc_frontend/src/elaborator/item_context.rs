//! The elaborator state that belongs to the item currently being elaborated, and the swap that
//! keeps one item's elaboration from observing or changing another's.
//!
//! Elaborating an item can require elaborating a different item first. For example, a reference to
//! a function returning `impl Trait` needs that function's body to learn the concrete type, so the
//! callee is elaborated in the middle of the caller's body. The callee must start from its own
//! context, and the caller must get its context back unchanged: otherwise the caller goes on
//! resolving `Self`, generics, trait bounds, loops and `unsafe` blocks as if it were the callee.
//!
//! [`Elaborator::with_item_context`] installs a whole [`ItemContext`] for the duration of a closure
//! and reinstates the previous one afterwards. Since the state lives in one struct that is swapped
//! as a unit, a field added here is saved and restored by construction.
//!
//! Within that struct, fields which are only meaningful together live in a sub-context of their
//! own - [`ModuleContext`] for which item is being elaborated and where it lives, [`ImplContext`]
//! for the enclosing impl or trait, [`GenericsContext`] for the generics and bounds in scope,
//! [`BodyContext`] for where in the item's body the elaborator is. Each sub-context owns the
//! operations over its own fields, so a caller that needs one group does not get a handle on the
//! rest.

use super::{Elaborator, types::ImplTraitDisallowedContext};

mod body_context;
mod generics_context;
mod impl_context;
mod module_context;

pub(crate) use body_context::BodyContext;
pub(crate) use generics_context::GenericsContext;
pub(crate) use impl_context::ImplContext;
pub(crate) use module_context::ModuleContext;

/// The elaborator state describing one item's elaboration.
///
/// This is the state that belongs to an item rather than to the crate being elaborated: anything
/// here is expected to start fresh for an item and to be irrelevant once that item is done. State
/// shared by the whole elaboration - the interner, the collected errors, the recursion depth - stays
/// on the [`Elaborator`] itself.
pub(super) struct ItemContext {
    /// The item being elaborated, and the module it was written in.
    pub(super) module: ModuleContext,

    /// The impl or trait the item belongs to, if any.
    pub(super) impl_context: ImplContext,

    /// The generics in scope, and the trait bounds they carry.
    pub(super) generics: GenericsContext,

    /// Where in the item's body the elaborator is.
    pub(super) body: BodyContext,

    /// True if we're elaborating a comptime item such as a comptime function,
    /// block, global, or attribute.
    pub(super) in_comptime_context: bool,

    /// Set when resolving types in positions where `impl Trait` is not allowed
    /// (e.g., struct fields, globals, type aliases, enum variants).
    /// `impl Trait` is only valid in function parameter and return type positions.
    ///
    /// This is stored as a field rather than checked at the call site so that it
    /// propagates through recursive `resolve_type` calls.
    pub(super) impl_trait_is_disallowed: Option<ImplTraitDisallowedContext>,
}

impl ItemContext {
    /// The context of an item described by `module` that is outside any impl or trait, has no
    /// generics in scope and is not comptime. An entry point whose item has any of those layers
    /// them on with the chained setters below.
    pub(super) fn new(module: ModuleContext) -> Self {
        Self {
            module,
            impl_context: ImplContext::default(),
            generics: GenericsContext::default(),
            body: BodyContext::default(),
            in_comptime_context: false,
            impl_trait_is_disallowed: None,
        }
    }

    /// The item belongs to the impl or trait described by `impl_context`.
    pub(super) fn with_impl(mut self, impl_context: ImplContext) -> Self {
        self.impl_context = impl_context;
        self
    }

    /// The item has `generics` in scope.
    pub(super) fn with_generics(mut self, generics: GenericsContext) -> Self {
        self.generics = generics;
        self
    }

    /// Whether the item is a comptime one.
    pub(super) fn in_comptime(mut self, in_comptime_context: bool) -> Self {
        self.in_comptime_context = in_comptime_context;
        self
    }

    /// Types resolved for the item may not mention `impl Trait`, because they sit in `context`.
    pub(super) fn disallowing_impl_trait(mut self, context: ImplTraitDisallowedContext) -> Self {
        self.impl_trait_is_disallowed = Some(context);
        self
    }
}

impl Elaborator<'_> {
    /// Runs `f` with `context` installed, then reinstates the context that was active before.
    ///
    /// Whatever `f` leaves in the context is discarded, so an entry point that elaborates another
    /// item through this helper cannot leak that item's state back to its caller.
    pub(super) fn with_item_context<T>(
        &mut self,
        context: ItemContext,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let outer = std::mem::replace(&mut self.item, context);
        let result = f(self);
        self.item = outer;
        result
    }
}
