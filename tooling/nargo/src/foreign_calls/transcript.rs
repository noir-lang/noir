use std::{collections::VecDeque, path::Path};

use acvm::{AcirField, acir::brillig::ForeignCallResult, pwg::ForeignCallWaitInfo};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{ForeignCallError, ForeignCallExecutor};

#[derive(Debug, thiserror::Error)]
pub enum TranscriptError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    DeserializationError(#[from] serde_json::Error),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
struct LogItem<F> {
    call: ForeignCallWaitInfo<F>,
    result: ForeignCallResult<F>,
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
}

impl<W, E, F> ForeignCallExecutor<F> for LoggingForeignCallExecutor<W, E>
where
    W: std::io::Write,
    F: AcirField + Serialize,
    E: ForeignCallExecutor<F>,
{
    fn execute(
        &mut self,
        foreign_call: &ForeignCallWaitInfo<F>,
    ) -> Result<ForeignCallResult<F>, ForeignCallError> {
        let result = self.executor.execute(foreign_call);
        if let Ok(ref result) = result {
            let log_item = || {
                // Match the JSON structure of `LogItem` without having to clone.
                let json = json!({"call": foreign_call, "result": result});
                serde_json::to_string(&json).expect("failed to serialize foreign call")
            };
            writeln!(self.output, "{}", log_item()).expect("write should succeed");
        }
        result
    }
}

/// Replay an oracle transcript which was logged with [LoggingForeignCallExecutor].
///
/// This is expected to be the last executor in the stack, e.g. prints can be handled above it.
pub struct ReplayForeignCallExecutor<F> {
    transcript: VecDeque<LogItem<F>>,
}

impl<F: for<'a> Deserialize<'a>> ReplayForeignCallExecutor<F> {
    pub fn from_file(path: &Path) -> Result<Self, TranscriptError> {
        let contents = std::fs::read_to_string(path)?;

        let transcript =
            contents.lines().map(serde_json::from_str).collect::<Result<VecDeque<_>, _>>()?;

        Ok(Self { transcript })
    }
}

impl<F> ForeignCallExecutor<F> for ReplayForeignCallExecutor<F>
where
    F: AcirField,
{
    fn execute(
        &mut self,
        foreign_call: &ForeignCallWaitInfo<F>,
    ) -> Result<ForeignCallResult<F>, ForeignCallError> {
        let error = |msg| Err(ForeignCallError::TranscriptError(msg));
        // Verify without popping.
        if let Some(next) = self.transcript.front() {
            if next.call.function != foreign_call.function {
                let msg = format!(
                    "unexpected foreign call; expected '{}', got '{}'",
                    next.call.function, foreign_call.function
                );
                return error(msg);
            }
            if next.call.inputs != foreign_call.inputs {
                let msg = format!(
                    "unexpected foreign call inputs to '{}'; expected {:?}, got {:?}",
                    next.call.function, next.call.inputs, foreign_call.inputs
                );
                return error(msg);
            }
        }
        // Consume the next call.
        if let Some(next) = self.transcript.pop_front() {
            Ok(next.result)
        } else {
            error("unexpected foreign call; no more calls in transcript".to_string())
        }
    }
}
