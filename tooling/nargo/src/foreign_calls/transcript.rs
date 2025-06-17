use std::path::Path;
use super::{ForeignCallError, ForeignCallExecutor};

#[derive(Debug, thiserror::Error)]
pub enum TranscriptError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    DeserializationError(#[from] serde_json::Error),
}

/// Log foreign calls during the execution, for testing purposes.
pub struct LoggingForeignCallExecutor<W, E> {
    pub executor: E,
    pub output: W,
}

impl<W, E> LoggingForeignCallExecutor<W, E> {
    pub fn new(executor: E, output: W) -> Self {
        Self { executor, output }
    }

    pub fn from_executor(executor: E) -> Self 
    where
        W: Default,
    {
        Self {
            executor,
            output: W::default(),
        }
    }
}

impl<W, E, F> ForeignCallExecutor<F> for LoggingForeignCallExecutor<W, E>
where
    W: std::io::Write,
    E: ForeignCallExecutor<F>,
{
    fn execute(
        &mut self,
        foreign_call: &str,
        inputs: &[F],
    ) -> Result<Vec<F>, ForeignCallError> {
        // Just forward to the inner executor in the stub
        self.executor.execute(foreign_call, inputs)
    }
}

/// Stub replay foreign call executor
pub struct ReplayForeignCallExecutor<F> {
    _phantom: std::marker::PhantomData<F>,
}

impl<F> ReplayForeignCallExecutor<F> {
    pub fn from_file(_path: &Path) -> Result<Self, TranscriptError> {
        Ok(Self {
            _phantom: std::marker::PhantomData,
        })
    }
}

impl<F> ForeignCallExecutor<F> for ReplayForeignCallExecutor<F> {
    fn execute(
        &mut self,
        foreign_call: &str,
        _inputs: &[F],
    ) -> Result<Vec<F>, ForeignCallError> {
        Err(ForeignCallError::TranscriptError(
            format!("Transcript replay not available without ACVM: {}", foreign_call)
        ))
    }
}