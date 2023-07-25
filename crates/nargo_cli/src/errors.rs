use acvm::{
    acir::native_types::WitnessMapError, Backend, CommonReferenceString, ProofSystemCompiler,
    SmartContract,
};
use hex::FromHexError;
use nargo::NargoError;
use noirc_abi::errors::{AbiError, InputParserError};
use noirc_errors::reporter::ReportedErrors;
use std::path::PathBuf;
use thiserror::Error;

use crate::resolver::DependencyResolutionError;

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

    #[error("Error: cannot remove file at {0:?}")]
    CanNotRemoveFile(PathBuf),

    /// Input parsing error
    #[error(transparent)]
    InputParserError(#[from] InputParserError),

    /// WitnessMap serialization error
    #[error(transparent)]
    WitnessMapSerialization(#[from] WitnessMapError),
}

#[derive(Debug, Error)]
pub(crate) enum CliError<B: Backend> {
    #[error("{0}")]
    Generic(String),
    #[error("Error: destination {} already exists", .0.display())]
    DestinationAlreadyExists(PathBuf),

    #[error("Failed to verify proof {}", .0.display())]
    InvalidProof(PathBuf),

    #[error(transparent)]
    ResolutionError(#[from] DependencyResolutionError),

    /// Errors encountered while compiling the noir program.
    /// These errors are already written to stderr.
    #[error("Aborting due to {} previous error{}", .0.error_count, if .0.error_count == 1 { "" } else { "s" })]
    ReportedErrors(ReportedErrors),

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

    /// Backend error caused by a function on the SmartContract trait
    #[error(transparent)]
    SmartContractError(<B as SmartContract>::Error), // Unfortunately, Rust won't let us `impl From` over an Associated Type on a generic

    /// Backend error caused by a function on the ProofSystemCompiler trait
    #[error(transparent)]
    ProofSystemCompilerError(<B as ProofSystemCompiler>::Error), // Unfortunately, Rust won't let us `impl From` over an Associated Type on a generic

    /// Backend error caused by a function on the CommonReferenceString trait
    #[error(transparent)]
    CommonReferenceStringError(<B as CommonReferenceString>::Error), // Unfortunately, Rust won't let us `impl From` over an Associated Type on a generic
}

impl<B: Backend> From<ReportedErrors> for CliError<B> {
    fn from(errors: ReportedErrors) -> Self {
        Self::ReportedErrors(errors)
    }
}
