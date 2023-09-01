use acvm::acir::native_types::WitnessMapError;
use acvm_backend_barretenberg::BackendError;
use hex::FromHexError;
use nargo::NargoError;
use nargo_toml::ManifestError;
use noirc_abi::errors::{AbiError, InputParserError};
use noirc_errors::reporter::ReportedErrors;
use noirc_frontend::graph::CrateName;
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

    /// Error from Nargo
    #[error(transparent)]
    NargoError(#[from] NargoError),

    /// Error from Manifest
    #[error(transparent)]
    ManifestError(#[from] ManifestError),

    /// Error from the compilation pipeline
    #[error(transparent)]
    CompileError(#[from] CompileError),

    /// Backend error
    #[error(transparent)]
    SmartContractError(#[from] BackendError),
}

/// Errors covering situations where a package cannot be compiled.
#[derive(Debug, Error)]
pub(crate) enum CompileError {
    #[error("Package `{0}` has type `lib` but only `bin` types can be compiled")]
    LibraryCrate(CrateName),

    #[error("Package `{0}` is expected to have a `main` function but it does not")]
    MissingMainFunction(CrateName),

    /// Errors encountered while compiling the Noir program.
    /// These errors are already written to stderr.
    #[error("Aborting due to {} previous error{}", .0.error_count, if .0.error_count == 1 { "" } else { "s" })]
    ReportedErrors(ReportedErrors),
}

impl From<ReportedErrors> for CompileError {
    fn from(errors: ReportedErrors) -> Self {
        Self::ReportedErrors(errors)
    }
}
