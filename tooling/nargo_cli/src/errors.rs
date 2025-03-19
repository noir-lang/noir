use acvm::{FieldElement, acir::native_types::WitnessStackError};
use hex::FromHexError;
use nargo::{NargoError, errors::CompileError};
use nargo_toml::ManifestError;
use noir_debugger::errors::DapError;
use noirc_abi::errors::{AbiError, InputParserError};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum FilesystemError {
    #[error("Error: {} is not a valid path\nRun either `nargo compile` to generate missing build artifacts or `nargo prove` to construct a proof", .0.display())]
    PathNotValid(PathBuf),

    #[error(
        "Error: could not parse hex build artifact (proof, proving and/or verification keys, ACIR checksum) ({0})"
    )]
    HexArtifactNotValid(FromHexError),

    #[error(
        " Error: cannot find {0}.toml file.\n Expected location: {1:?} \n Please generate this file at the expected location."
    )]
    MissingTomlFile(String, PathBuf),

    /// Input parsing error
    #[error(transparent)]
    InputParserError(#[from] InputParserError),

    /// WitnessStack serialization error
    #[error(transparent)]
    WitnessStackSerialization(#[from] WitnessStackError),
}

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
}

impl From<FilesystemError> for CliError {
    fn from(error: FilesystemError) -> Self {
        match error {
            FilesystemError::PathNotValid(ref _path) => CliError::Generic(format!("{}", error)),
            FilesystemError::HexArtifactNotValid(ref _e) => CliError::Generic(format!("{}", error)),
            FilesystemError::MissingTomlFile(ref _name, ref _path) => {
                CliError::Generic(format!("{}", error))
            }
            FilesystemError::InputParserError(ref _e) => CliError::Generic(format!("{}", error)),
            FilesystemError::WitnessStackSerialization(ref _e) => {
                CliError::Generic(format!("{}", error))
            }
        }
    }
}
