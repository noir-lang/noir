//! The part of an item's context that only a function body can have: the loop, `unsafe` block,
//! lambda and call-argument scopes an expression sits in.

use crate::elaborator::{LambdaContext, Loop};

/// Determines whether we are in an unsafe block and, if so, whether
/// any unconstrained calls were found in it (because if not we'll warn
/// that the unsafe block is not needed).
#[derive(Copy, Clone, Default)]
enum UnsafeBlockStatus {
    #[default]
    NotInUnsafeBlock,
    InUnsafeBlockWithoutUnconstrainedCalls,
    InUnsafeBlockWithUnconstrainedCalls,
}

/// The `unsafe` block state of the code around the block just entered, to be handed back to
/// [`BodyContext::exit_unsafe_block`] when it is left.
pub(crate) struct EnclosingUnsafeBlock(UnsafeBlockStatus);

impl EnclosingUnsafeBlock {
    /// Whether the block just entered is itself inside an `unsafe` block, which makes it
    /// redundant.
    pub(crate) fn is_nested(&self) -> bool {
        !matches!(self.0, UnsafeBlockStatus::NotInUnsafeBlock)
    }
}

/// Where in a function body the elaborator currently is.
///
/// Every field here belongs to a scope nested inside the item - a loop, an `unsafe` block, a
/// lambda, a call's arguments - so each is entered and left in pairs, rather than set once for
/// the item. An item with no body leaves this [`Default`].
#[derive(Default)]
pub(crate) struct BodyContext {
    current_loop: Option<Loop>,

    /// When resolving lambda expressions, we need to keep track of the variables
    /// that are captured. We do this in order to create the hidden environment
    /// parameter for the lambda function.
    lambda_stack: Vec<LambdaContext>,

    unsafe_block_status: UnsafeBlockStatus,

    /// True if we are elaborating arguments of a function call to an unconstrained function.
    in_unconstrained_args: bool,

    /// If greater than 0, field visibility errors won't be reported.
    /// This is used when elaborating a comptime expression that is a struct constructor
    /// like `Foo { inner: 5 }`: in that case we already elaborated the code that led to
    /// that comptime value and any visibility errors were already reported.
    silence_field_visibility_errors: usize,

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
    lvalue_index_counter: usize,
}

impl BodyContext {
    /// Enters a loop body, returning the loop that was in scope so it can be handed back to
    /// [`Self::exit_loop`].
    #[must_use]
    pub(crate) fn enter_loop(&mut self, is_for: bool) -> Option<Loop> {
        self.current_loop.replace(Loop { is_for, has_break: false })
    }

    /// Leaves a loop body, restoring `outer` and returning the loop that was just left.
    pub(crate) fn exit_loop(&mut self, outer: Option<Loop>) -> Loop {
        std::mem::replace(&mut self.current_loop, outer).expect("Expected a loop")
    }

    /// The innermost loop the elaborator is in, if any.
    pub(crate) fn current_loop_mut(&mut self) -> Option<&mut Loop> {
        self.current_loop.as_mut()
    }

    /// Enters an `unsafe` block, returning the status of the enclosing code so it can be handed
    /// back to [`Self::exit_unsafe_block`].
    #[must_use]
    pub(crate) fn enter_unsafe_block(&mut self) -> EnclosingUnsafeBlock {
        EnclosingUnsafeBlock(std::mem::replace(
            &mut self.unsafe_block_status,
            UnsafeBlockStatus::InUnsafeBlockWithoutUnconstrainedCalls,
        ))
    }

    /// Leaves an `unsafe` block, returning whether it contained an unconstrained call.
    ///
    /// The `enclosing` status is reinstated unless this block is nested in another `unsafe`
    /// block and contained an unconstrained call, in which case the enclosing block is
    /// considered to contain that call as well.
    pub(crate) fn exit_unsafe_block(&mut self, enclosing: EnclosingUnsafeBlock) -> bool {
        let has_unconstrained_call = matches!(
            self.unsafe_block_status,
            UnsafeBlockStatus::InUnsafeBlockWithUnconstrainedCalls
        );
        if !enclosing.is_nested() || !has_unconstrained_call {
            self.unsafe_block_status = enclosing.0;
        }
        has_unconstrained_call
    }

    /// Records that the code being elaborated calls an unconstrained function from a constrained
    /// one, and reports whether it is allowed to: such a call must be in an `unsafe` block, which
    /// is in turn justified by containing it.
    pub(crate) fn note_unconstrained_call(&mut self) -> bool {
        match self.unsafe_block_status {
            UnsafeBlockStatus::NotInUnsafeBlock => false,
            UnsafeBlockStatus::InUnsafeBlockWithoutUnconstrainedCalls => {
                self.unsafe_block_status = UnsafeBlockStatus::InUnsafeBlockWithUnconstrainedCalls;
                true
            }
            UnsafeBlockStatus::InUnsafeBlockWithUnconstrainedCalls => true,
        }
    }

    /// Enters the arguments of a call to a function that is unconstrained or not, returning the
    /// enclosing call's status so it can be handed back to [`Self::exit_call_arguments`].
    #[must_use]
    pub(crate) fn enter_call_arguments(&mut self, unconstrained: bool) -> bool {
        std::mem::replace(&mut self.in_unconstrained_args, unconstrained)
    }

    /// Leaves a call's arguments, reinstating the enclosing call's status.
    pub(crate) fn exit_call_arguments(&mut self, enclosing_unconstrained: bool) {
        self.in_unconstrained_args = enclosing_unconstrained;
    }

    /// Whether the expression being elaborated is an argument of a call to an unconstrained
    /// function.
    pub(crate) fn in_unconstrained_args(&self) -> bool {
        self.in_unconstrained_args
    }

    /// Enters a lambda body, whose captures are collected until [`Self::exit_lambda`].
    pub(crate) fn enter_lambda(&mut self, scope_index: usize, unconstrained: bool) {
        self.lambda_stack.push(LambdaContext { captures: Vec::new(), scope_index, unconstrained });
    }

    /// Leaves a lambda body, returning what it captured.
    pub(crate) fn exit_lambda(&mut self) -> LambdaContext {
        self.lambda_stack.pop().expect("Expected a lambda")
    }

    /// The lambda whose body is being elaborated, if any. Lambdas nest, so this is the innermost.
    pub(crate) fn current_lambda(&self) -> Option<&LambdaContext> {
        self.lambda_stack.last()
    }

    /// How many lambdas deep the elaborator is: a variable declared outside them is captured once
    /// per level on its way in.
    pub(crate) fn lambda_depth(&self) -> usize {
        self.lambda_stack.len()
    }

    /// The lambda `index` levels in from the outermost one whose body is being elaborated.
    pub(crate) fn lambda_at_depth_mut(&mut self, index: usize) -> &mut LambdaContext {
        &mut self.lambda_stack[index]
    }

    /// Stops reporting field visibility errors until [`Self::unsilence_field_visibility_errors`].
    pub(crate) fn silence_field_visibility_errors(&mut self) {
        self.silence_field_visibility_errors += 1;
    }

    /// Reports field visibility errors again, undoing one
    /// [`Self::silence_field_visibility_errors`].
    pub(crate) fn unsilence_field_visibility_errors(&mut self) {
        self.silence_field_visibility_errors -= 1;
    }

    pub(crate) fn field_visibility_errors_silenced(&self) -> bool {
        self.silence_field_visibility_errors > 0
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn next_lvalue_index_counter(&mut self) -> usize {
        let lvalue_index_counter = self.lvalue_index_counter;
        self.lvalue_index_counter += 1;
        lvalue_index_counter
    }
}
