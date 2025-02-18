use acvm::FieldElement;
use nargo::{errors::CompileError, NargoError};
use nargo_toml::ManifestError;
use noir_debugger::errors::DapError;
use noirc_abi::{errors::AbiError, input_parser::InputValue, AbiReturnType};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum CliError {
    #[error("{0}")]
    Generic(String),

    #[error("Error: destination {} already exists", .0.display())]
    DestinationAlreadyExists(PathBuf),

    #[error("Invalid package name {0}. Did you mean to use `--name`?")]
    InvalidPackageName(String),

    /// Artifact CLI error
    #[error(transparent)]
    ArtifactError(#[from] noir_artifact_cli::errors::CliError),

    /// ABI encoding/decoding error
    #[error(transparent)]
    AbiError(#[from] AbiError),

    #[error(transparent)]
    LspError(#[from] async_lsp::Error),

    #[error(transparent)]
    DapError(#[from] DapError),

    /// Error from Nargo
    #[error(transparent)]
    NargoError(#[from] NargoError<FieldElement>),

    /// Error from Manifest
    #[error(transparent)]
    ManifestError(#[from] ManifestError),

    /// Error from the compilation pipeline
    #[error(transparent)]
    CompileError(#[from] CompileError),

    #[error("Unexpected return value: expected {expected:?}; got {actual:?}")]
    UnexpectedReturn { expected: InputValue, actual: Option<InputValue> },

    #[error("Missing return witnesses; expected {expected:?}")]
    MissingReturn { expected: AbiReturnType },
}
