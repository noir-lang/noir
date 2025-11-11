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

/// Interface for executing foreign calls
pub trait ForeignCallExecutor<F> {
    fn execute(
        &mut self,
        foreign_call: &ForeignCallWaitInfo<F>,
    ) -> Result<ForeignCallResult<F>, ForeignCallError>;
}

/// This enumeration represents the Brillig foreign calls that are natively supported by nargo.
/// After resolution of a foreign call, nargo will restart execution of the ACVM
pub enum ForeignCall {
    /// Reference [mod@print] for more info regarding this call's inputs
    Print,
    CreateMock,
    SetMockParams,
    GetMockLastParams,
    SetMockReturns,
    SetMockTimes,
    ClearMock,
    GetTimesCalled,
}

impl std::fmt::Display for ForeignCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl ForeignCall {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            ForeignCall::Print => "print",
            ForeignCall::CreateMock => "create_mock",
            ForeignCall::SetMockParams => "set_mock_params",
            ForeignCall::GetMockLastParams => "get_mock_last_params",
            ForeignCall::SetMockReturns => "set_mock_returns",
            ForeignCall::SetMockTimes => "set_mock_times",
            ForeignCall::ClearMock => "clear_mock",
            ForeignCall::GetTimesCalled => "get_times_called",
        }
    }

    pub(crate) fn lookup(op_name: &str) -> Option<ForeignCall> {
        match op_name {
            "print" => Some(ForeignCall::Print),
            "create_mock" => Some(ForeignCall::CreateMock),
            "set_mock_params" => Some(ForeignCall::SetMockParams),
            "get_mock_last_params" => Some(ForeignCall::GetMockLastParams),
            "set_mock_returns" => Some(ForeignCall::SetMockReturns),
            "set_mock_times" => Some(ForeignCall::SetMockTimes),
            "clear_mock" => Some(ForeignCall::ClearMock),
            "get_times_called" => Some(ForeignCall::GetTimesCalled),
            _ => None,
        }
    }
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

    #[error(
        "Oracle call `{0}` requires connection connection to an Aztec simulation environment. See https://foo.bar/aztec for more information."
    )]
    AztecOracleError(String),
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

#[cfg(test)]
mod tests {
    use super::ForeignCallError;
    use crate::foreign_calls::{DefaultForeignCallBuilder, ForeignCallExecutor};

    #[test]
    /// We special-case oracle calls with the "aztec_" prefix to return a specific error in the 
    /// case where they are unhandled. This test ensures that this behavior is preserved. 
    fn throws_correct_error_for_unhandled_aztec_oracle() {
        use acvm::{FieldElement, acir::brillig::ForeignCallResult, pwg::ForeignCallWaitInfo};

        let mut executor = DefaultForeignCallBuilder::default()
            .build::<FieldElement>();
        let foreign_call =
            ForeignCallWaitInfo { function: "aztec_get_value".to_string(), inputs: vec![] };

        let result: Result<ForeignCallResult<FieldElement>, ForeignCallError> =
            executor.execute(&foreign_call);

        match result {
            Err(ForeignCallError::AztecOracleError(func_name)) => {
                assert_eq!(func_name, "aztec_get_value");
            }
            _ => panic!("Expected AztecOracleError"),
        }
    }
}
