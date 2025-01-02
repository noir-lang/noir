use acvm::{acir::brillig::ForeignCallResult, pwg::ForeignCallWaitInfo, AcirField};

use super::{ForeignCallError, ForeignCallExecutor};

/// Returns an empty result when called.
///
/// If all executors have no handler for the given foreign call then we cannot
/// return a correct response to the ACVM. The best we can do is to return an empty response,
/// this allows us to ignore any foreign calls which exist solely to pass information from inside
/// the circuit to the environment (e.g. custom logging) as the execution will still be able to progress.
///
/// We optimistically return an empty response for all oracle calls as the ACVM will error
/// should a response have been required.
pub struct Empty;

impl<F: AcirField> ForeignCallExecutor<F> for Empty {
    fn execute(
        &mut self,
        _foreign_call: &ForeignCallWaitInfo<F>,
    ) -> Result<ForeignCallResult<F>, ForeignCallError> {
        Ok(ForeignCallResult::default())
    }
}

/// Returns `NoHandler` for every call.
pub struct Unhandled;

impl<F: AcirField> ForeignCallExecutor<F> for Unhandled {
    fn execute(
        &mut self,
        foreign_call: &ForeignCallWaitInfo<F>,
    ) -> Result<ForeignCallResult<F>, ForeignCallError> {
        Err(ForeignCallError::NoHandler(foreign_call.function.clone()))
    }
}

/// Forwards to the inner executor if its own handler doesn't handle the call.
pub struct Layer<H, I> {
    pub handler: H,
    pub inner: I,
}

impl<H, I, F> ForeignCallExecutor<F> for Layer<H, I>
where
    H: ForeignCallExecutor<F>,
    I: ForeignCallExecutor<F>,
{
    fn execute(
        &mut self,
        foreign_call: &ForeignCallWaitInfo<F>,
    ) -> Result<ForeignCallResult<F>, ForeignCallError> {
        match self.handler.execute(foreign_call) {
            Err(ForeignCallError::NoHandler(_)) => self.inner.execute(foreign_call),
            handled => handled,
        }
    }
}

impl<H, I> Layer<H, I> {
    /// Create a layer from two handlers
    pub fn new(handler: H, inner: I) -> Self {
        Self { handler, inner }
    }
}

impl<H> Layer<H, Empty> {
    /// Create a layer from a handler.
    /// If the handler doesn't handle a call, a default empty response is returned.
    pub fn or_empty(handler: H) -> Self {
        Self { handler, inner: Empty }
    }
}

impl<H> Layer<H, Unhandled> {
    /// Create a layer from a handler.
    /// If the handler doesn't handle a call, `NoHandler` error is returned.
    pub fn or_unhandled(handler: H) -> Self {
        Self { handler, inner: Unhandled }
    }
}

impl Layer<Unhandled, Unhandled> {
    /// A base layer that doesn't handle anything.
    pub fn unhandled() -> Self {
        Self { handler: Unhandled, inner: Unhandled }
    }
}

impl<H, I> Layer<H, I> {
    /// Add another layer on top of this one.
    pub fn add_layer<J>(self, handler: J) -> Layer<J, Self> {
        Layer::new(handler, self)
    }

    pub fn handler(&self) -> &H {
        &self.handler
    }

    pub fn inner(&self) -> &I {
        &self.inner
    }
}

/// Compose handlers.
pub trait Layering {
    /// Layer an executor on top of this one.
    /// The `other` executor will be called first.
    fn add_layer<L, F>(self, other: L) -> Layer<L, Self>
    where
        Self: Sized + ForeignCallExecutor<F>,
        L: ForeignCallExecutor<F>;
}

impl<T> Layering for T {
    fn add_layer<L, F>(self, other: L) -> Layer<L, T>
    where
        T: Sized + ForeignCallExecutor<F>,
        L: ForeignCallExecutor<F>,
    {
        Layer::new(other, self)
    }
}

/// Support disabling a layer by making it optional.
/// This way we can still have a known static type for a composition,
/// because layers are always added, potentially wrapped in an `Option`.
impl<H, F> ForeignCallExecutor<F> for Option<H>
where
    H: ForeignCallExecutor<F>,
{
    fn execute(
        &mut self,
        foreign_call: &ForeignCallWaitInfo<F>,
    ) -> Result<ForeignCallResult<F>, ForeignCallError> {
        match self {
            Some(handler) => handler.execute(foreign_call),
            None => Err(ForeignCallError::NoHandler(foreign_call.function.clone())),
        }
    }
}
