use std::path::PathBuf;

use acvm::{acir::brillig::ForeignCallResult, pwg::ForeignCallWaitInfo, AcirField};
use serde::Serialize;
use serde_json::json;

use crate::PrintOutput;

use super::{ForeignCallError, ForeignCallExecutor};

/// Log foreign calls during the execution, for testing purposes.
pub struct LogForeignCallExecutor<'a, E> {
    pub executor: E,
    pub output: PrintOutput<'a>,
}

impl<'a, E> LogForeignCallExecutor<'a, E> {
    pub fn new(executor: E, output: PrintOutput<'a>) -> Self {
        Self { executor, output }
    }
}

impl<'a, E, F> ForeignCallExecutor<F> for LogForeignCallExecutor<'a, E>
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
