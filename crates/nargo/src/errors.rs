use acvm::pwg::OpcodeResolutionError;
use noirc_printable_type::ForeignCallError;
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
}
