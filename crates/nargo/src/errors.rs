use acvm::OpcodeResolutionError;
use hex::FromHexError;
use noirc_abi::errors::{AbiError, InputParserError};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum CliError {
    #[error("{0}")]
    Generic(String),
    #[error("Error: destination {} already exists", .0.display())]
    DestinationAlreadyExists(PathBuf),
    #[error("Error: {} is not a valid path\nRun either `nargo compile` to generate missing build artifacts or `nargo prove` to construct a proof", .0.display())]
    PathNotValid(PathBuf),
    #[error("Error: could not parse hex build artifact (proof, proving and/or verification keys, ACIR checksum) ({0})")]
    HexArtifactNotValid(FromHexError),
    #[error(
        " Error: cannot find {0}.toml file.\n Expected location: {1:?} \n Please generate this file at the expected location."
    )]
    MissingTomlFile(String, PathBuf),
    #[error("Error: the circuit you are trying to prove differs from the build artifact at {}\nYou must call `nargo compile` to generate the correct proving and verification keys for this circuit", .0.display())]
    MismatchedAcir(PathBuf),
    #[error("Failed to verify proof {}", .0.display())]
    InvalidProof(PathBuf),

    /// Error while compiling Noir into ACIR.
    #[error("Failed to compile circuit")]
    CompilationError,

    /// Input parsing error
    #[error(transparent)]
    InputParserError(#[from] InputParserError),

    /// ABI encoding/decoding error
    #[error(transparent)]
    AbiError(#[from] AbiError),

    /// ACIR circuit solving error
    #[error(transparent)]
    SolvingError(#[from] OpcodeResolutionError),
}
