use nargo::NargoError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum FilesystemError {
    #[error(
        " Error: cannot find {0} in expected location {1:?}.\n Please generate this file at the expected location."
    )]
    MissingTomlFile(String, PathBuf),
    #[error(" Error: failed to parse toml file {0}.")]
    InvalidTomlFile(String),
    #[error(
      " Error: cannot find {0} in expected location {1:?}.\n Please generate this file at the expected location."
    )]
    MissingBytecodeFile(String, PathBuf),

    #[error(" Error: failed to read bytecode file {0}.")]
    InvalidBytecodeFile(String),

    #[error(" Error: failed to create output witness file {0}.")]
    OutputWitnessCreationFailed(String),

    #[error(" Error: failed to write output witness file {0}.")]
    OutputWitnessWriteFailed(String),
}

#[derive(Debug, Error)]
pub(crate) enum CliError {
    /// Filesystem errors
    #[error(transparent)]
    FilesystemError(#[from] FilesystemError),

    /// Error related to circuit deserialization
    #[error("Error: failed to deserialize circuit")]
    CircuitDeserializationError(),

    /// Error related to circuit execution
    #[error(transparent)]
    CircuitExecutionError(#[from] NargoError),

    /// Input Witness Value Error
    #[error("Error: failed to parse witness value {0}")]
    WitnessValueError(String),

    /// Input Witness Index Error
    #[error("Error: failed to parse witness index {0}")]
    WitnessIndexError(String),

    #[error(" Error: failed to serialize output witness.")]
    OutputWitnessSerializationFailed(),
}
