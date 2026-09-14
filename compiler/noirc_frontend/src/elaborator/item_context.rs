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
//! and reinstates the previous one afterwards, so an entry point cannot restore some of these
//! fields and forget others.

use crate::{
    Type,
    hir::def_map::LocalModuleId,
    hir_def::{traits::TraitConstraint, types::ResolvedGeneric},
    node_interner::{DependencyId, ImplId, TraitId, TraitImplId},
};

use super::{Elaborator, LambdaContext, Loop, UnsafeBlockStatus};

/// Per-item elaborator state. Each field mirrors the [`Elaborator`] field of the same name.
pub(super) struct ItemContext {
    pub(super) local_module: Option<LocalModuleId>,
    pub(super) current_item: Option<DependencyId>,
    pub(super) self_type: Option<Type>,
    pub(super) current_trait: Option<TraitId>,
    pub(super) current_trait_impl: Option<TraitImplId>,
    pub(super) current_impl: Option<ImplId>,
    pub(super) generics: Vec<ResolvedGeneric>,
    pub(super) trait_bounds: Vec<TraitConstraint>,
    pub(super) lambda_stack: Vec<LambdaContext>,
    pub(super) current_loop: Option<Loop>,
    pub(super) unsafe_block_status: UnsafeBlockStatus,
    pub(super) in_comptime_context: bool,
    pub(super) in_unconstrained_args: bool,
    pub(super) silence_field_visibility_errors: usize,
    pub(super) lvalue_index_counter: usize,
}

impl Elaborator<'_> {
    /// Runs `f` with `context` installed, then reinstates the context that was active before.
    ///
    /// Whatever `f` leaves in these fields is discarded, so an entry point that elaborates
    /// another item through this helper cannot leak that item's state back to its caller.
    pub(super) fn with_item_context<T>(
        &mut self,
        context: ItemContext,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let outer = self.replace_item_context(context);
        let result = f(self);
        self.replace_item_context(outer);
        result
    }

    /// Installs `context` and returns the context it replaced.
    fn replace_item_context(&mut self, context: ItemContext) -> ItemContext {
        // Destructuring without `..` makes adding a field to `ItemContext` a compile error here
        // until the new field is swapped as well.
        let ItemContext {
            local_module,
            current_item,
            self_type,
            current_trait,
            current_trait_impl,
            current_impl,
            generics,
            trait_bounds,
            lambda_stack,
            current_loop,
            unsafe_block_status,
            in_comptime_context,
            in_unconstrained_args,
            silence_field_visibility_errors,
            lvalue_index_counter,
        } = context;

        ItemContext {
            local_module: std::mem::replace(&mut self.local_module, local_module),
            current_item: std::mem::replace(&mut self.current_item, current_item),
            self_type: std::mem::replace(&mut self.self_type, self_type),
            current_trait: std::mem::replace(&mut self.current_trait, current_trait),
            current_trait_impl: std::mem::replace(&mut self.current_trait_impl, current_trait_impl),
            current_impl: std::mem::replace(&mut self.current_impl, current_impl),
            generics: std::mem::replace(&mut self.generics, generics),
            trait_bounds: std::mem::replace(&mut self.trait_bounds, trait_bounds),
            lambda_stack: std::mem::replace(&mut self.lambda_stack, lambda_stack),
            current_loop: std::mem::replace(&mut self.current_loop, current_loop),
            unsafe_block_status: std::mem::replace(
                &mut self.unsafe_block_status,
                unsafe_block_status,
            ),
            in_comptime_context: std::mem::replace(
                &mut self.in_comptime_context,
                in_comptime_context,
            ),
            in_unconstrained_args: std::mem::replace(
                &mut self.in_unconstrained_args,
                in_unconstrained_args,
            ),
            silence_field_visibility_errors: std::mem::replace(
                &mut self.silence_field_visibility_errors,
                silence_field_visibility_errors,
            ),
            lvalue_index_counter: std::mem::replace(
                &mut self.lvalue_index_counter,
                lvalue_index_counter,
            ),
        }
    }
}
