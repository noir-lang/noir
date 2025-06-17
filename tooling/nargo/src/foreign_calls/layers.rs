use super::{ForeignCallError, ForeignCallExecutor};

/// Returns an empty result when called.
pub struct Empty;

impl<F> ForeignCallExecutor<F> for Empty {
    fn execute(
        &mut self,
        _foreign_call: &str,
        _inputs: &[F],
    ) -> Result<Vec<F>, ForeignCallError> {
        Ok(Vec::new())
    }
}

/// Returns `NoHandler` for every call.
pub struct Unhandled;

impl<F> ForeignCallExecutor<F> for Unhandled {
    fn execute(
        &mut self,
        foreign_call: &str,
        _inputs: &[F],
    ) -> Result<Vec<F>, ForeignCallError> {
        Err(ForeignCallError::NoHandler(foreign_call.to_string()))
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
        foreign_call: &str,
        inputs: &[F],
    ) -> Result<Vec<F>, ForeignCallError> {
        match self.handler.execute(foreign_call, inputs) {
            Err(ForeignCallError::NoHandler(_)) => self.inner.execute(foreign_call, inputs),
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

/// A case where we can have either this or that type of handler.
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R, F> ForeignCallExecutor<F> for Either<L, R>
where
    L: ForeignCallExecutor<F>,
    R: ForeignCallExecutor<F>,
{
    fn execute(
        &mut self,
        foreign_call: &str,
        inputs: &[F],
    ) -> Result<Vec<F>, ForeignCallError> {
        match self {
            Either::Left(left) => left.execute(foreign_call, inputs),
            Either::Right(right) => right.execute(foreign_call, inputs),
        }
    }
}

/// Support disabling a layer by making it optional.
impl<H, F> ForeignCallExecutor<F> for Option<H>
where
    H: ForeignCallExecutor<F>,
{
    fn execute(
        &mut self,
        foreign_call: &str,
        inputs: &[F],
    ) -> Result<Vec<F>, ForeignCallError> {
        match self {
            Some(handler) => handler.execute(foreign_call, inputs),
            None => Err(ForeignCallError::NoHandler(foreign_call.to_string())),
        }
    }
}