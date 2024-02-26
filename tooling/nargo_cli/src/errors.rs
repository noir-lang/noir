use acvm::acir::native_types::WitnessMapError;
use hex::FromHexError;
use nargo::{errors::CompileError, NargoError};
use nargo_toml::ManifestError;
use noir_debugger::errors::DapError;
use noirc_abi::errors::{AbiError, InputParserError};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum FilesystemError {
    #[error("Error: {} is not a valid path\nRun either `nargo compile` to generate missing build artifacts or `nargo prove` to construct a proof", .0.display())]
    PathNotValid(PathBuf),
    #[error("Error: could not parse hex build artifact (proof, proving and/or verification keys, ACIR checksum) ({0})")]
    HexArtifactNotValid(FromHexError),
    #[error(
        " Error: cannot find {0}.toml file.\n Expected location: {1:?} \n Please generate this file at the expected location."
    )]
    MissingTomlFile(String, PathBuf),

    /// Input parsing error
    #[error(transparent)]
    InputParserError(#[from] InputParserError),

    /// WitnessMap serialization error
    #[error(transparent)]
    WitnessMapSerialization(#[from] WitnessMapError),

    #[error("Error: could not deserialize build program: {0}")]
    ProgramSerializationError(String),
}

#[derive(Debug, Error)]
pub(crate) enum CliError {
    #[error("{0}")]
    Generic(String),
    #[error("Error: destination {} already exists", .0.display())]
    DestinationAlreadyExists(PathBuf),

    #[error("Failed to verify proof {}", .0.display())]
    InvalidProof(PathBuf),

    #[error("Invalid package name {0}. Did you mean to use `--name`?")]
    InvalidPackageName(String),

    /// ABI encoding/decoding error
    #[error(transparent)]
    AbiError(#[from] AbiError),

    /// Filesystem errors
    #[error(transparent)]
    FilesystemError(#[from] FilesystemError),

    #[error(transparent)]
    LspError(#[from] async_lsp::Error),

    #[error(transparent)]
    DapError(#[from] DapError),

    /// Error from Nargo
    #[error(transparent)]
    NargoError(#[from] NargoError),

    /// Error from Manifest
    #[error(transparent)]
    ManifestError(#[from] ManifestError),

    /// Error from the compilation pipeline
    #[error(transparent)]
    CompileError(#[from] CompileError),

    /// Error related to backend selection/installation.
    #[error(transparent)]
    BackendError(#[from] BackendError),

    /// Error related to communication with backend.
    #[error(transparent)]
    BackendCommunicationError(#[from] backend_interface::BackendError),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum BackendError {
    #[error("No backend is installed with the name {0}")]
    UnknownBackend(String),

    #[error("The backend {0} is already installed")]
    AlreadyInstalled(String),

    #[error("Backend installation failed: {0}")]
    InstallationError(#[from] std::io::Error),
}
