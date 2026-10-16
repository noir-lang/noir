//! The monomorphizer state that belongs to the function currently being monomorphized.
//!
//! The [`Monomorphizer`] runs over a whole program, but most of what it tracks while lowering an
//! expression only makes sense within one function: which HIR definitions map to which locals,
//! the closure environment, whether the code is unconstrained, and the bindings for the
//! function's generics. That state lives in a [`FunctionContext`], and
//! [`Monomorphizer::with_function_context`] installs a fresh one for each function taken off the
//! queue, so nothing one function leaves behind is visible to the next. State that spans the
//! whole program - the queue, the finished functions and globals, the ID counters - stays on the
//! [`Monomorphizer`] itself.
//!
//! A lambda body is a function too, and [`Monomorphizer::with_lambda_context`] gives it a context
//! of its own. That context starts with only the lambda's parameters as locals: everything else
//! the body reads from the enclosing function is a capture, reached through the lambda's
//! environment. The one thing it shares with the enclosing function is the generic bindings,
//! since the lambda's types mention the enclosing function's generics.

use crate::TypeBindings;
use crate::hir_def::expr::HirCapturedVar;
use crate::node_interner;
use rustc_hash::FxHashMap as HashMap;

use super::Monomorphizer;
use super::ast::{self, LocalId};

/// The closure environment of a lambda whose body is being monomorphized.
#[derive(Clone)]
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

    /// The environment holding the captures of the closure being monomorphized, if this is a
    /// closure body.
    pub(super) lambda_env: Option<LambdaContext>,

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
            lambda_env: None,
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
    /// leak locals, closure environments or generic bindings into the next one. The only thing
    /// kept is the function's locals, for [`Monomorphizer::locals`] to report.
    pub(super) fn with_function_context<T>(
        &mut self,
        context: FunctionContext,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let outer = std::mem::replace(&mut self.function, context);
        let result = f(self);
        let inner = std::mem::replace(&mut self.function, outer);
        self.last_function_locals = inner.locals;
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

    /// Runs `f` with the locals that `f` defines collected into a map of their own, which is
    /// returned alongside `f`'s result. The current context's locals are untouched.
    ///
    /// This is how a lambda's parameters are defined: they belong to the lambda's context, which
    /// [`Self::with_lambda_context`] then installs for the lambda's body.
    pub(super) fn collecting_locals<T>(
        &mut self,
        f: impl FnOnce(&mut Self) -> T,
    ) -> (T, HashMap<node_interner::DefinitionId, LocalId>) {
        let outer = std::mem::take(&mut self.function.locals);
        let result = f(self);
        let collected = std::mem::replace(&mut self.function.locals, outer);
        (result, collected)
    }

    /// Runs `f` to monomorphize the body of a lambda, in a context of the lambda's own.
    ///
    /// The context has `locals` (the lambda's parameters) as its only locals, `lambda_env` as its
    /// closure environment if the lambda has captures, and is unconstrained if `unconstrained` is
    /// set. It borrows the enclosing context's generic bindings for the duration, then hands them
    /// back.
    pub(super) fn with_lambda_context<T>(
        &mut self,
        locals: HashMap<node_interner::DefinitionId, LocalId>,
        lambda_env: Option<LambdaContext>,
        unconstrained: bool,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let context = FunctionContext {
            locals,
            lambda_env,
            in_unconstrained_function: unconstrained,
            force_unconstrained: self.force_brillig,
            substitution: std::mem::take(&mut self.function.substitution),
        };
        let outer = std::mem::replace(&mut self.function, context);
        let result = f(self);
        let inner = std::mem::replace(&mut self.function, outer);
        self.function.substitution = inner.substitution;
        result
    }
}
