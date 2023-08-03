use acvm::{
    acir::native_types::WitnessMapError, Backend, CommonReferenceString, ProofSystemCompiler,
    SmartContract,
};
use hex::FromHexError;
use nargo::{package::PackageType, NargoError};
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
pub(crate) enum CliError<B: Backend> {
    #[error("{0}")]
    Generic(String),
    #[error("Error: destination {} already exists", .0.display())]
    DestinationAlreadyExists(PathBuf),

    #[error("Failed to verify proof {}", .0.display())]
    InvalidProof(PathBuf),

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

/// Errors covering situations where a package is either missing or malformed.
#[derive(Debug, Error)]
pub(crate) enum ManifestError {
    /// Package doesn't have a manifest file
    #[error("cannot find a Nargo.toml in {}", .0.display())]
    MissingFile(PathBuf),

    #[error("Cannot read file {0} - does it exist?")]
    ReadFailed(PathBuf),

    #[error("Nargo.toml is missing a parent directory")]
    MissingParent,

    #[error("Missing `type` field in {0}")]
    MissingPackageType(PathBuf),

    #[error("Cannot use `{1}` for `type` field in {0}")]
    InvalidPackageType(PathBuf, String),

    /// Package manifest is unreadable.
    #[error("Nargo.toml is badly formed, could not parse.\n\n {0}")]
    MalformedFile(#[from] toml::de::Error),

    #[error("Unxpected workspace definition found in {0}")]
    UnexpectedWorkspace(PathBuf),

    #[error("Cannot find file {entry} which was specified as the `entry` field in {toml}")]
    MissingEntryFile { toml: PathBuf, entry: PathBuf },

    #[error(
        r#"Cannot find file {entry} which is defaulted due to specifying `type = "{package_type}"` in {toml}"#
    )]
    MissingDefaultEntryFile { toml: PathBuf, entry: PathBuf, package_type: PackageType },

    /// Invalid character `-` in package name
    #[error("invalid character `-` in package name")]
    InvalidPackageName,

    /// Encountered error while downloading git repository.
    #[error("{0}")]
    GitError(String),

    #[error("Selected package `{0}` was not found")]
    MissingSelectedPackage(CrateName),

    #[error("Default package was not found. Does {0} exist in your workspace?")]
    MissingDefaultPackage(PathBuf),

    #[error("Package `{0}` has type `bin` but you cannot depend on binary packages")]
    BinaryDependency(CrateName),
}
