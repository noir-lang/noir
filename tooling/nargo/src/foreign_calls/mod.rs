use thiserror::Error;

/// Error when trying to convert foreign call parameters
#[derive(Debug, Error)]
pub enum TryFromParamsError {
    #[error("Foreign call inputs needed for execution are missing")]
    MissingForeignCallInputs,
    #[error("Could not parse parameters: {0}")]
    ParsingError(#[from] serde_json::Error),
}

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

/// Interface for executing foreign calls (stubbed without ACVM)
pub trait ForeignCallExecutor<F> {
    fn execute(
        &mut self,
        foreign_call: &str,
        inputs: &[F],
    ) -> Result<Vec<F>, ForeignCallError>;
}

/// This enumeration represents the Brillig foreign calls that are natively supported by nargo.
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

    #[error("Other error: {0}")]
    Other(String),
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