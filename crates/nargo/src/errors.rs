use acvm::pwg::OpcodeResolutionError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NargoError {
    #[error("{0}")]
    Generic(String),

    /// Error while compiling Noir into ACIR.
    #[error("Failed to compile circuit")]
    CompilationError,

    /// ACIR circuit solving error
    #[error(transparent)]
    SolvingError(#[from] OpcodeResolutionError),
}
