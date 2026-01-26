use acvm::{acir::brillig::ForeignCallResult, pwg::ForeignCallWaitInfo};
use noirc_printable_type::TryFromParamsError;
use thiserror::Error;

pub mod layers;
pub mod mocker;
pub mod print;
pub mod transcript;

pub mod default;
#[cfg(feature = "rpc")]
pub mod rpc;
pub use default::DefaultForeignCallBuilder;
#[cfg(feature = "rpc")]
pub use default::DefaultForeignCallExecutor;

pub use noirc_frontend::shared::ForeignCall;

/// Interface for executing foreign calls
pub trait ForeignCallExecutor<F> {
    fn execute(
        &mut self,
        foreign_call: &ForeignCallWaitInfo<F>,
    ) -> Result<ForeignCallResult<F>, ForeignCallError>;
}

#[derive(Debug, Error)]
pub enum ForeignCallError {
    #[error("Attempted to call disabled foreign call `{0}`")]
    Disabled(String),

    #[error("No handler could be found for foreign call `{0}`")]
    NoHandler(String),

    #[error("Foreign call inputs needed for execution are missing")]
    MissingForeignCallInputs,

    #[error("Could not parse PrintableType argument. {0}")]
    ParsingError(#[from] serde_json::Error),

    #[error("Failed calling external resolver. {0}")]
    ExternalResolverError(#[from] jsonrpsee::core::client::Error),

    #[error("Assert message resolved after an unsatisfied constrain. {0}")]
    ResolvedAssertMessage(String),

    #[error("Failed to replay oracle transcript: {0}")]
    TranscriptError(String),
}

impl From<TryFromParamsError> for ForeignCallError {
    fn from(err: TryFromParamsError) -> Self {
        match err {
            TryFromParamsError::MissingForeignCallInputs => {
                ForeignCallError::MissingForeignCallInputs
            }
            TryFromParamsError::ParsingError(error) => ForeignCallError::ParsingError(error),
        }
    }
}
