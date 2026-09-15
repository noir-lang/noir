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

use std::collections::BTreeSet;

use crate::{
    Type,
    hir::def_map::{LocalModuleId, ModuleId},
    hir_def::{traits::TraitConstraint, types::ResolvedGeneric},
    node_interner::{DependencyId, ImplId, TraitId, TraitImplId},
};

use super::{
    Elaborator, LambdaContext, Loop, UnsafeBlockStatus, types::ImplTraitDisallowedContext,
};

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

    /// Set to the current type if we're resolving an impl
    pub(super) self_type: Option<Type>,

    /// The trait we're currently resolving or implementing, if any.
    /// Set during both trait definitions (`trait Foo { ... }`) and
    /// trait impl elaboration (`impl Foo for Bar { ... }`).
    pub(super) current_trait: Option<TraitId>,

    /// If we're currently resolving methods within a trait impl, this will be set
    /// to the corresponding trait impl ID.
    pub(super) current_trait_impl: Option<TraitImplId>,

    /// If we're currently resolving methods within an inherent (non-trait) impl,
    /// this will be set to the corresponding impl ID.
    pub(super) current_impl: Option<ImplId>,

    /// Contains a mapping of the current struct or functions's generics to
    /// unique type variables if we're resolving a struct. Empty otherwise.
    /// This is a Vec rather than a map to preserve the order a functions generics
    /// were declared in.
    pub(super) generics: Vec<ResolvedGeneric>,

    /// Each constraint in the `where` clause of the function currently being resolved.
    pub(super) trait_bounds: Vec<TraitConstraint>,

    /// Every `(object type, trait)` pair brought into scope by implication rather than by a
    /// `where` clause naming it: a bound declared on an associated type, or a parent trait.
    ///
    /// A written bound which duplicates one of these is not reported as unnecessary. The two can
    /// be registered in either order - the implication may come from a later clause in the same
    /// `where` list - so remembering the implied pairs is what makes the diagnostic independent of
    /// that order. Emptied together with the assumed impls by
    /// [`Elaborator::remove_trait_constraints_from_scope`].
    pub(super) implied_trait_bounds: BTreeSet<(Type, TraitId)>,

    /// When resolving lambda expressions, we need to keep track of the variables
    /// that are captured. We do this in order to create the hidden environment
    /// parameter for the lambda function.
    pub(super) lambda_stack: Vec<LambdaContext>,

    pub(super) current_loop: Option<Loop>,

    pub(super) unsafe_block_status: UnsafeBlockStatus,

    /// True if we're elaborating a comptime item such as a comptime function,
    /// block, global, or attribute.
    pub(super) in_comptime_context: bool,

    /// True if we are elaborating arguments of a function call to an unconstrained function.
    pub(super) in_unconstrained_args: bool,

    /// Set when resolving types in positions where `impl Trait` is not allowed
    /// (e.g., struct fields, globals, type aliases, enum variants).
    /// `impl Trait` is only valid in function parameter and return type positions.
    ///
    /// This is stored as a field rather than checked at the call site so that it
    /// propagates through recursive `resolve_type` calls.
    pub(super) impl_trait_is_disallowed: Option<ImplTraitDisallowedContext>,

    /// If greater than 0, field visibility errors won't be reported.
    /// This is used when elaborating a comptime expression that is a struct constructor
    /// like `Foo { inner: 5 }`: in that case we already elaborated the code that led to
    /// that comptime value and any visibility errors were already reported.
    pub(super) silence_field_visibility_errors: usize,

    /// Counter used to define temporary variables for non-simple indexes in l-values.
    ///
    /// For example, this expression:
    ///
    /// ```noir
    /// array[x + y] = 10;
    /// ```
    ///
    /// is transformed into:
    ///
    /// ```noir
    /// let i_0 = x + y;
    /// array[i_0] = 10;
    /// ```
    pub(super) lvalue_index_counter: usize,
}

impl ItemContext {
    /// The type `Self` refers to. Only call this where an impl, trait or trait impl context is
    /// installed; items that have no `Self` leave this unset.
    pub(super) fn self_type(&self) -> &Type {
        self.self_type.as_ref().expect("self_type is unset")
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn next_lvalue_index_counter(&mut self) -> usize {
        let lvalue_index_counter = self.lvalue_index_counter;
        self.lvalue_index_counter += 1;
        lvalue_index_counter
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
