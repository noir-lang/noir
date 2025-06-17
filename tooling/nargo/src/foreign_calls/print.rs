use std::io::Write;
use super::{ForeignCall, ForeignCallError, ForeignCallExecutor};

/// Handle `println` calls.
#[derive(Debug, Default)]
pub struct PrintForeignCallExecutor<W> {
    output: W,
}

impl<W> PrintForeignCallExecutor<W> {
    pub fn new(output: W) -> Self {
        Self { output }
    }
}

impl<F, W: Write> ForeignCallExecutor<F> for PrintForeignCallExecutor<W> {
    fn execute(
        &mut self,
        foreign_call: &str,
        _inputs: &[F],
    ) -> Result<Vec<F>, ForeignCallError> {
        if let Some(ForeignCall::Print) = ForeignCall::lookup(foreign_call) {
            // Without ACVM, we can't properly decode the print parameters
            // Just write a placeholder message
            writeln!(self.output, "[Print call - content unavailable without ACVM]")
                .map_err(|_| ForeignCallError::Other("Failed to write to output".to_string()))?;
            Ok(Vec::new())
        } else {
            Err(ForeignCallError::NoHandler(foreign_call.to_string()))
        }
    }
}