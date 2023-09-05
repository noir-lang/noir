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

#[derive(Debug, Error)]
pub enum ExecutionError {
    #[error("Failed assertion: '{}'", .0)]
    AssertionFailed(String, OpcodeLocation),

    #[error(transparent)]
    SolvingError(#[from] OpcodeResolutionError),
}
