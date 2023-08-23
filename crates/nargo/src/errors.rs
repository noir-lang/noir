use acvm::pwg::{ErrorLocation, OpcodeResolutionError};
use noirc_abi::errors::{AbiError, InputParserError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NargoError {
    /// Error while compiling Noir into ACIR.
    #[error("Failed to compile circuit")]
    CompilationError,

    /// ACIR circuit solving errors
    #[error("{}", format_unsatisfied_constraint(.0, .1))]
    UnsatisfiedConstrain(Option<String>, ErrorLocation),

    #[error(transparent)]
    SolvingError(#[from] OpcodeResolutionError),

    /// Oracle handling error
    #[error(transparent)]
    ForeignCallError(#[from] ForeignCallError),
}

fn format_unsatisfied_constraint(
    assert_message: &Option<String>,
    error_location: &ErrorLocation,
) -> String {
    match (assert_message, error_location) {
        (Some(message), _) => {
            format!("Failed assertion: {}", message)
        }
        (None, ErrorLocation::Resolved(opcode_location)) => {
            format!("Unsatisfied constraint at {}", opcode_location)
        }
        _ => "Unsatisfied constraint".to_owned(),
    }
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
