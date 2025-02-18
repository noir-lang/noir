use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
};

use acvm::{acir::brillig::ForeignCallResult, pwg::ForeignCallWaitInfo, AcirField};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::PrintOutput;

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
pub struct LoggingForeignCallExecutor<'a, E> {
    pub executor: E,
    pub output: PrintOutput<'a>,
}

impl<'a, E> LoggingForeignCallExecutor<'a, E> {
    pub fn new(executor: E, output: PrintOutput<'a>) -> Self {
        Self { executor, output }
    }
}

impl<'a, E, F> ForeignCallExecutor<F> for LoggingForeignCallExecutor<'a, E>
where
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
            match &mut self.output {
                PrintOutput::None => (),
                PrintOutput::Stdout => println!("{}", log_item()),
                PrintOutput::String(s) => {
                    s.push_str(&log_item());
                    s.push('\n');
                }
            }
        }
        result
    }
}

/// Log foreign calls to stdout as soon as soon as they are made, or buffer them and write to a file at the end.
pub enum ForeignCallLog {
    None,
    Stdout,
    File(PathBuf, String),
}

impl ForeignCallLog {
    /// Instantiate based on an env var.
    pub fn from_env(key: &str) -> Self {
        match std::env::var(key) {
            Err(_) => Self::None,
            Ok(s) if s == "stdout" => Self::Stdout,
            Ok(s) => Self::File(PathBuf::from(s), String::new()),
        }
    }

    /// Create a [PrintOutput] based on the log setting.
    pub fn print_output(&mut self) -> PrintOutput {
        match self {
            ForeignCallLog::None => PrintOutput::None,
            ForeignCallLog::Stdout => PrintOutput::Stdout,
            ForeignCallLog::File(_, s) => PrintOutput::String(s),
        }
    }

    /// Any final logging.
    pub fn write_log(self) -> std::io::Result<()> {
        if let ForeignCallLog::File(path, contents) = self {
            std::fs::write(path, contents)?;
        }
        Ok(())
    }
}

/// Replay an oracle transcript which was logged with [LoggingForeignCallExecutor].
///
/// This is expected to be the last executor in the stack, e.g. prints can be handled above it.
pub struct TranscriptForeignCallExecutor<F> {
    transcript: VecDeque<LogItem<F>>,
}

impl<F: for<'a> Deserialize<'a>> TranscriptForeignCallExecutor<F> {
    pub fn from_file(path: &Path) -> Result<Self, TranscriptError> {
        let contents = std::fs::read_to_string(path)?;

        let transcript =
            contents.lines().map(serde_json::from_str).collect::<Result<VecDeque<_>, _>>()?;

        Ok(Self { transcript })
    }
}

impl<F> ForeignCallExecutor<F> for TranscriptForeignCallExecutor<F>
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
