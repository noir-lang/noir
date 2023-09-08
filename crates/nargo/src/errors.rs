use acvm::{acir::circuit::OpcodeLocation, pwg::OpcodeResolutionError};
use noirc_printable_type::ForeignCallError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NargoError {
    /// Error while compiling Noir into ACIR.
    #[error("Failed to compile circuit")]
    CompilationError,

    /// ACIR circuit execution error
    #[error(transparent)]
    ExecutionError(#[from] ExecutionError),

    /// Oracle handling error
    #[error(transparent)]
    ForeignCallError(#[from] ForeignCallError),
}

impl NargoError {
    /// Extracts the user defined failure message from the ExecutionError
    /// If one exists.
    ///
    /// We want to extract the user defined error so that we can compare it
    /// in tests to expected failure messages
    pub fn user_defined_failure_message(&self) -> Option<&str> {
        let execution_error = match self {
            NargoError::ExecutionError(error) => error,
            _ => return None,
        };

        match execution_error {
            ExecutionError::AssertionFailed(message, _) => Some(message),
            ExecutionError::SolvingError(error) => match error {
                OpcodeResolutionError::IndexOutOfBounds { .. }
                | OpcodeResolutionError::UnsupportedBlackBoxFunc(_)
                | OpcodeResolutionError::OpcodeNotSolvable(_)
                | OpcodeResolutionError::UnsatisfiedConstrain { .. } => None,
                OpcodeResolutionError::BrilligFunctionFailed { message, .. } => Some(message),
                OpcodeResolutionError::BlackBoxFunctionFailed(_, reason) => Some(reason),
            },
        }
    }
}

#[derive(Debug, Error)]
pub enum ExecutionError {
    #[error("Failed assertion: '{}'", .0)]
    AssertionFailed(String, Vec<OpcodeLocation>),

    #[error(transparent)]
    SolvingError(#[from] OpcodeResolutionError),
}
