//! The elaborator state shared by every [`Elaborator`] taking part in one elaboration.
//!
//! Comptime code can ask for more elaboration in the middle of being interpreted: unquoting a
//! type or module, expanding a macro call, or lazily elaborating the body of a function it is
//! about to call. Each of these runs in a fresh [`Elaborator`] so that the item being elaborated
//! by the parent is not disturbed (see [`Elaborator::elaborate_item_from_comptime`]).
//!
//! A fresh elaborator is only fresh with respect to its item. The limits, flags and pending work
//! of the surrounding comptime evaluation must carry through it, or a nested elaboration could
//! reset a recursion limit, clear a halt, or lose deferred items. That state lives in
//! [`ElaborationSession`], which the fresh elaborator takes over for its lifetime and hands back
//! when it is done. Because the struct moves as a unit, a field added here is shared by
//! construction.
//!
//! Per-elaborator state that is deliberately *not* shared - the scope forest, the function
//! context stack, the in-resolution type ids and the collected errors - stays on the
//! [`Elaborator`] itself, and the item being elaborated lives in its
//! [`ItemContext`](super::item_context::ItemContext).

use noirc_errors::Location;
use rustc_hash::FxHashSet as HashSet;

use crate::hir::comptime::ComptimeScopes;

use super::{ElaborateReason, Elaborator, deferred::DeferredItems};

/// State that spans every [`Elaborator`] taking part in one elaboration, including the fresh
/// elaborators created to elaborate items on behalf of comptime code.
#[derive(Default)]
pub(crate) struct ElaborationSession {
    /// Locations of the comptime calls currently being interpreted, outermost first.
    pub(super) interpreter_call_stack: imbl::Vector<Location>,

    /// Sometimes items are elaborated because a function attribute ran and generated items.
    /// The Elaborator keeps track of these reasons so that when an error is produced it will
    /// be wrapped in another error that will include this reason.
    pub(super) elaborate_reasons: imbl::Vector<ElaborateReason>,

    /// Set to true when the interpreter encounters an errored expression/statement,
    /// causing all subsequent comptime evaluation to be skipped.
    pub(super) comptime_evaluation_halted: bool,

    /// Tracks the current macro expansion depth to prevent infinite recursion
    /// when an attribute generates code that triggers further attribute expansion.
    /// This catches both single-function and mutual recursion, including recursion
    /// that passes through a nested elaborator.
    pub(super) macro_expansion_depth: usize,

    /// Current expression nesting depth, counted across nested elaborators since they
    /// all share the same native stack.
    pub(super) recursion_depth: usize,

    /// Variable names from enclosing runtime scopes, used for error reporting only.
    /// While a nested elaborator runs, this holds the names of the variables visible in every
    /// elaborator that encloses it. If a variable lookup fails and the name is in this set, we
    /// can report a more specific error about runtime variables not being available in comptime
    /// code.
    pub(super) parent_runtime_variables: HashSet<String>,

    /// Items registered for resolution later, and the trait bookkeeping that waits on them.
    /// See the [`deferred`](super::deferred) module for why each kind is deferred and what the
    /// drains guarantee.
    pub(super) deferred: DeferredItems,

    /// Local variables of the comptime code being interpreted, together with the comptime
    /// variables of the runtime blocks being elaborated. A nested elaborator sees the variables
    /// visible to the comptime function that asked for it.
    pub(super) comptime_scopes: ComptimeScopes,
}

impl Elaborator<'_> {
    /// Whether comptime evaluation has been halted by an earlier error.
    pub(crate) fn comptime_evaluation_halted(&self) -> bool {
        self.session.comptime_evaluation_halted
    }

    /// Skip all further comptime evaluation in this elaboration.
    pub(crate) fn halt_comptime_evaluation(&mut self) {
        self.session.comptime_evaluation_halted = true;
    }

    pub(crate) fn comptime_scopes(&self) -> &ComptimeScopes {
        &self.session.comptime_scopes
    }

    pub(crate) fn comptime_scopes_mut(&mut self) -> &mut ComptimeScopes {
        &mut self.session.comptime_scopes
    }
}
