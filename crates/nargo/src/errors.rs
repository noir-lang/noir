use acvm::{acir::circuit::OpcodeLocation, pwg::OpcodeResolutionError};
use noirc_abi::errors::{AbiError, InputParserError};
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

#[derive(Debug, Error)]
pub enum ForeignCallError {
    #[error("Foreign call inputs needed for execution are missing")]
    MissingForeignCallInputs,

    /// ABI encoding/decoding error
    #[error(transparent)]
    AbiError(#[from] AbiError),

    /// Input parsing error
    #[error(transparent)]
    InputParserError(#[from] InputParserError),
}
