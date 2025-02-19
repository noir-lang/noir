use acir::FieldElement;
use nargo::NargoError;
use noirc_abi::{
    errors::{AbiError, InputParserError},
    input_parser::InputValue,
    AbiReturnType,
};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FilesystemError {
    #[error("Cannot find input file '{0}'")]
    MissingInputFile(PathBuf),

    #[error("Failed to parse input file '{0}': {1}")]
    InvalidInputFile(PathBuf, String),

    #[error("Cannot find bytecode file '{0}'")]
    MissingBytecodeFile(PathBuf),

    #[error("Failed to read bytecode file '{0}': {1}")]
    InvalidBytecodeFile(PathBuf, String),

    #[error("Failed to create output witness file '{0}': {1}")]
    OutputWitnessCreationFailed(PathBuf, String),

    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum CliError {
    /// Filesystem errors
    #[error(transparent)]
    FilesystemError(#[from] FilesystemError),

    /// Error related to ABI input deserialization
    #[error("Failed to deserialize inputs")]
    InputDeserializationError(#[from] InputParserError),

    /// Error related to ABI encoding
    #[error(transparent)]
    AbiError(#[from] AbiError),

    /// Error related to artifact deserialization
    #[error("Failed to deserialize artifact from JSON")]
    ArtifactDeserializationError(#[from] serde_json::Error),

    /// Error related to circuit deserialization
    #[error("Failed to deserialize circuit from bytecode")]
    CircuitDeserializationError(#[from] std::io::Error),

    /// Error related to circuit execution
    #[error(transparent)]
    CircuitExecutionError(#[from] NargoError<FieldElement>),

    /// Input Witness Value Error
    #[error("Failed to parse witness value {0}")]
    WitnessValueError(String),

    /// Input Witness Index Error
    #[error("Failed to parse witness index {0}")]
    WitnessIndexError(String),

    #[error("Failed to serialize output witness: {0}")]
    OutputWitnessSerializationFailed(#[from] toml::ser::Error),

    #[error("Unexpected return value: expected {expected:?}; got {actual:?}")]
    UnexpectedReturn { expected: InputValue, actual: Option<InputValue> },

    #[error("Missing return witnesses; expected {expected:?}")]
    MissingReturn { expected: AbiReturnType },

    #[error("Missing contract function name; options: {names:?}")]
    MissingContractFn { names: Vec<String> },

    #[error("Unknown contract function '{name}'; options: {names:?}")]
    UnknownContractFn { name: String, names: Vec<String> },
}
