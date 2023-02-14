use acvm::OpcodeResolutionError;
use hex::FromHexError;
use noirc_abi::errors::{AbiError, InputParserError};
use std::{io::Write, path::PathBuf};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("{0}")]
    Generic(String),
    #[error("Error: destination {} already exists", .0.display())]
    DestinationAlreadyExists(PathBuf),
    #[error("Error: {} is not a valid path", .0.display())]
    PathNotValid(PathBuf),
    #[error("Error: could not parse proof data ({0})")]
    ProofNotValid(FromHexError),
    #[error(
        " Error: cannot find {0}.toml file.\n Expected location: {1:?} \n Please generate this file at the expected location."
    )]
    MissingTomlFile(String, PathBuf),
    #[error("Error: cannot find proving key located at {}\nEither run nargo compile to generate the missing proving key or check that the correct file name has been provided", .0.display())]
    MissingProvingKey(PathBuf),
    #[error("Error: cannot find verification key located at {}\nEither run nargo compile to generate the missing verification key or check that the correct file name has been provided", .0.display())]
    MissingVerificationkey(PathBuf),
}

impl From<OpcodeResolutionError> for CliError {
    fn from(value: OpcodeResolutionError) -> Self {
        CliError::Generic(value.to_string())
    }
}

impl CliError {
    pub(crate) fn write(&self) -> ! {
        let mut stderr = StandardStream::stderr(ColorChoice::Always);
        stderr
            .set_color(ColorSpec::new().set_fg(Some(Color::Red)))
            .expect("cannot set color for stderr in StandardStream");
        writeln!(&mut stderr, "{self}").expect("cannot write to stderr");

        std::process::exit(1)
    }
}

impl From<InputParserError> for CliError {
    fn from(error: InputParserError) -> Self {
        CliError::Generic(error.to_string())
    }
}

impl From<AbiError> for CliError {
    fn from(error: AbiError) -> Self {
        CliError::Generic(error.to_string())
    }
}
