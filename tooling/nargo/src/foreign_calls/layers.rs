use std::marker::PhantomData;

use acvm::{acir::brillig::ForeignCallResult, pwg::ForeignCallWaitInfo, AcirField};
use noirc_printable_type::ForeignCallError;

use super::ForeignCallExecutor;

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

/// Forwards to the inner executor if its own handler doesn't handle the call.
pub struct Layer<H, I, F> {
    handler: H,
    inner: I,
    _field: PhantomData<F>,
}

impl<H, I, F> ForeignCallExecutor<F> for Layer<H, I, F>
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

impl<H, F> Layer<H, Empty, F> {
    /// Create a layer from a handler.
    pub fn new(handler: H) -> Self {
        Layer { handler, inner: Empty, _field: PhantomData }
    }
}

impl<H, I, F> Layer<H, I, F> {
    /// Compose layers.
    #[allow(clippy::should_implement_trait)]
    pub fn add<J>(self, handler: J) -> Layer<J, Self, F> {
        Layer { handler, inner: self, _field: PhantomData }
    }

    pub fn handler(&self) -> &H {
        &self.handler
    }

    pub fn inner(&self) -> &I {
        &self.inner
    }
}

/// We can create an empty layer and compose on top of it;
/// the `inner` will never be called.
impl<F> Default for Layer<Empty, Empty, F> {
    fn default() -> Self {
        Self::new(Empty)
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
