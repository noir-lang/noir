use hex::FromHexError;
use noirc_abi::errors::{AbiError, InputParserError};
use std::{fmt::Display, io::Write, path::PathBuf};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

#[derive(Debug)]
pub enum CliError {
    Generic(String),
    DestinationAlreadyExists(PathBuf),
    PathNotValid(PathBuf),
    ProofNotValid(FromHexError),
    MissingTomlFile(PathBuf),
}

impl CliError {
    pub(crate) fn write(&self) -> ! {
        let mut stderr = StandardStream::stderr(ColorChoice::Always);
        stderr
            .set_color(ColorSpec::new().set_fg(Some(Color::Red)))
            .expect("cannot set color for stderr in StandardStream");
        writeln!(&mut stderr, "{}", self).expect("cannot write to stderr");

        std::process::exit(1)
    }
}

impl Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
                CliError::Generic(msg) => write!(f, "Error: {}", msg),
                CliError::DestinationAlreadyExists(path) =>
                write!(f, "Error: destination {} already exists", path.display()),
                CliError::PathNotValid(path) => {
                    write!(f, "Error: {} is not a valid path", path.display())
                }
                CliError::ProofNotValid(hex_error) => {
                    write!(f, "Error: could not parse proof data ({})", hex_error)
                }
                CliError::MissingTomlFile(path) => write!(f, "cannot find input file located at {:?}, run nargo build to generate the missing Prover and/or Verifier toml files", path),
            }
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
