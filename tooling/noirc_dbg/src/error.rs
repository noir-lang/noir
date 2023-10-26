use dap::errors::ServerError;
use thiserror::Error;

/// Wraps app specific errors.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Error)]
pub enum DebuggingError {
    /// ACIR circuit execution error
    #[error(transparent)]
    ExecutionError(#[from] nargo::errors::ExecutionError),

    /// Custom debugger error
    #[error("{0:?}")]
    CustomError(String),

    /// Dap server error
    #[error(transparent)]
    ServerError(ServerError),

    /// Bytecode handling error
    #[error(transparent)]
    ForeignCallError(#[from] noirc_printable_type::ForeignCallError),
}

impl From<ServerError> for DebuggingError {
    fn from(value: ServerError) -> Self {
        Self::ServerError(value)
    }
}
