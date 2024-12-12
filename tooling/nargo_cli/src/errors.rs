use acvm::{acir::native_types::WitnessStackError, FieldElement};
use nargo::{errors::CompileError, NargoError};
use nargo_toml::ManifestError;
use noir_debugger::errors::DapError;
use noirc_abi::{
    errors::{AbiError, InputParserError},
    input_parser::InputValue,
    AbiReturnType,
};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum FilesystemError {
    #[error("Error: {} is not a valid path\nRun either `nargo compile` to generate missing build artifacts or `nargo prove` to construct a proof", .0.display())]
    PathNotValid(PathBuf),

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

    #[error("Error: could not deserialize build program: {0}")]
    ProgramSerializationError(String),
}

#[derive(Debug, Error)]
pub(crate) enum CliError {
    #[error("{0}")]
    Generic(String),

    #[error("Error: destination {} already exists", .0.display())]
    DestinationAlreadyExists(PathBuf),

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
