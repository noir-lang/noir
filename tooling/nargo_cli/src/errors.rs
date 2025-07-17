use acvm::FieldElement;
use nargo::{NargoError, errors::CompileError};
use nargo_toml::ManifestError;
use noir_debugger::errors::DapError;
use noirc_abi::errors::AbiError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("{0}")]
    Generic(String),

    #[error("Error: destination {} already exists\n\nUse `new init` to initialize the directory", .0.display())]
    DestinationAlreadyExists(PathBuf),

    #[error(
        "Error: `nargo init` cannot be run on existing packages.\nNote: `Nargo.toml` already exists."
    )]
    NargoInitCannotBeRunOnExistingPackages,

    #[error(
        "Error: {0}\nIf you need a package name to not match the directory name, consider using the `--name` flag."
    )]
    InvalidPackageName(String),

    #[error("`--debug-compile-stdin` is incompatible with `--watch`")]
    CantWatchStdin,

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
