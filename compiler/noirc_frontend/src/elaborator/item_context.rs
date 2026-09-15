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
//! own - [`ImplContext`] for the enclosing impl or trait, [`GenericsContext`] for the generics and
//! bounds in scope, [`BodyContext`] for where in the item's body the elaborator is. Each
//! sub-context owns the operations over its own fields, so a caller that needs one group does not
//! get a handle on the rest.

use crate::{
    hir::def_map::{LocalModuleId, ModuleId},
    node_interner::DependencyId,
};

use super::{Elaborator, types::ImplTraitDisallowedContext};

mod body_context;
mod generics_context;
mod impl_context;

pub(crate) use body_context::BodyContext;
pub(crate) use generics_context::GenericsContext;
pub(crate) use impl_context::ImplContext;

/// The elaborator state describing one item's elaboration.
///
/// This is the state that belongs to an item rather than to the crate being elaborated: anything
/// here is expected to start fresh for an item and to be irrelevant once that item is done. State
/// shared by the whole elaboration - the interner, the collected errors, the recursion depth - stays
/// on the [`Elaborator`] itself.
#[derive(Default)]
pub(super) struct ItemContext {
    /// The current module this elaborator is in.
    /// Initially None, it is set whenever a new top-level item is resolved.
    pub(super) local_module: Option<LocalModuleId>,

    /// The current dependency item we're resolving.
    /// Used to link items to their dependencies in the dependency graph
    pub(super) current_item: Option<DependencyId>,

    /// When set, visibility checks during path resolution use this module
    /// instead of the default importing module.
    ///
    /// Set when resolving an expression on behalf of comptime code from another module (see
    /// `Expr::resolve`), so that the item resolved that way is held to the caller's visibility
    /// rather than to that of the scope it is resolved in.
    pub(super) caller_module: Option<ModuleId>,

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
