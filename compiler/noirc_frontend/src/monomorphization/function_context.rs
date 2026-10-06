//! The monomorphizer state that belongs to the function currently being monomorphized.
//!
//! The [`Monomorphizer`] runs over a whole program, but most of what it tracks while lowering an
//! expression only makes sense within one function: which HIR definitions map to which locals,
//! the closure environments in scope, whether the code is unconstrained, and the bindings for the
//! function's generics. That state lives in a [`FunctionContext`], and
//! [`Monomorphizer::with_function_context`] installs a fresh one for each function taken off the
//! queue, so nothing one function leaves behind is visible to the next. State that spans the
//! whole program - the queue, the finished functions and globals, the ID counters - stays on the
//! [`Monomorphizer`] itself.
//!
//! A lambda is not a function of its own here: its body is monomorphized inside the context of
//! the function that contains it, since it reads that function's locals and generics.

use crate::TypeBindings;
use crate::hir_def::expr::HirCapturedVar;
use crate::node_interner;
use rustc_hash::FxHashMap as HashMap;

use super::Monomorphizer;
use super::ast::{self, LocalId};

/// The closure environment of a lambda whose body is being monomorphized.
pub(super) struct LambdaContext {
    pub(super) env_ident: ast::Ident,
    pub(super) captures: Vec<HirCapturedVar>,
}

/// The monomorphizer state describing one function's monomorphization.
pub(super) struct FunctionContext {
    /// The monomorphized local created for each HIR definition in the function.
    ///
    /// Locals are only keyed by their unique ID because they are never duplicated during
    /// monomorphization. Doing so would allow them to be used polymorphically but would also
    /// cause them to be re-evaluated, which is a performance trap that would confuse users.
    pub(super) locals: HashMap<node_interner::DefinitionId, LocalId>,

    /// The environments of the closures being monomorphized, innermost last.
    pub(super) lambda_envs_stack: Vec<LambdaContext>,

    /// Whether the code being monomorphized is unconstrained. A constrained function called from
    /// unconstrained code is monomorphized as unconstrained too.
    pub(super) in_unconstrained_function: bool,

    /// Set while monomorphizing an expression whose target slot is typed `unconstrained fn(..)`.
    /// Note that this also changes the first-class function representation from a pair of
    /// `(constrained, unconstrained)` to `(unconstrained, unconstrained)`, so that a constrained
    /// caller dispatching through slot `.0` still runs the unconstrained version.
    ///
    /// This is a property of the position being monomorphized: it holds for the value stored into
    /// that slot and for nothing else. A binding nested inside that value's expression carries its
    /// own type and its own slot, so it is monomorphized under its own value of this field.
    pub(super) force_unconstrained: bool,

    /// Bindings for the generics of the function being monomorphized: the union of every set
    /// passed to a live [`Monomorphizer::with_bindings`] call. Every type the pass reads from the
    /// HIR goes through [`Monomorphizer::ty`], which applies them.
    pub(super) substitution: TypeBindings,
}

impl FunctionContext {
    /// The context of a function with no locals, closures or generic bindings yet, whose code is
    /// unconstrained if `in_unconstrained_function` is set.
    ///
    /// `force_brillig` is the program-wide `--force-brillig` setting, which forces every position
    /// in every function to be unconstrained.
    pub(super) fn new(in_unconstrained_function: bool, force_brillig: bool) -> Self {
        Self {
            locals: HashMap::default(),
            lambda_envs_stack: Vec::new(),
            in_unconstrained_function,
            force_unconstrained: force_brillig,
            substitution: TypeBindings::default(),
        }
    }
}

impl Monomorphizer<'_> {
    /// Runs `f` with `context` installed, then reinstates the context that was active before.
    ///
    /// Whatever `f` leaves in the context is discarded, so the function it monomorphizes cannot
    /// leak locals, closure environments or generic bindings into the next one.
    pub(super) fn with_function_context<T>(
        &mut self,
        context: FunctionContext,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let outer = std::mem::replace(&mut self.function, context);
        let result = f(self);
        self.function = outer;
        result
    }

    /// Runs `f` with [`FunctionContext::in_unconstrained_function`] set to `unconstrained`, then
    /// reinstates the previous value.
    pub(super) fn with_in_unconstrained_function<T>(
        &mut self,
        unconstrained: bool,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let outer = std::mem::replace(&mut self.function.in_unconstrained_function, unconstrained);
        let result = f(self);
        self.function.in_unconstrained_function = outer;
        result
    }

    /// Runs `f` with [`FunctionContext::force_unconstrained`] set to `forced`, then reinstates the
    /// previous value.
    pub(super) fn with_force_unconstrained<T>(
        &mut self,
        forced: bool,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let outer = std::mem::replace(&mut self.function.force_unconstrained, forced);
        let result = f(self);
        self.function.force_unconstrained = outer;
        result
    }

    /// Runs `f` with `lambda` as the innermost closure environment, then pops it.
    pub(super) fn with_lambda_env<T>(
        &mut self,
        lambda: LambdaContext,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        self.function.lambda_envs_stack.push(lambda);
        let result = f(self);
        self.function.lambda_envs_stack.pop();
        result
    }
}
