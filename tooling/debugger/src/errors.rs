use thiserror::Error;

#[derive(Debug, Error)]
pub enum DapError {
    #[error("{0}")]
    PreFlightGenericError(String),

    #[error(transparent)]
    LoadError(#[from] LoadError),

    #[error(transparent)]
    ServerError(#[from] dap::errors::ServerError),
}

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("{0}")]
    Generic(String),
}
