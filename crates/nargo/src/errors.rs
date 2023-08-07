use acvm::pwg::OpcodeResolutionError;
use noirc_abi::errors::{AbiError, InputParserError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NargoError {
    /// Error while compiling Noir into ACIR.
    #[error("Failed to compile circuit")]
    CompilationError,

    /// ACIR circuit solving error
    #[error(transparent)]
    SolvingError(#[from] OpcodeResolutionError),

    #[error(transparent)]
    ForeignCallError(#[from] ForeignCallError),

    #[error("Unsatisified constraint at index {0}")]
    UnsatisfiedConstraint(usize),
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
