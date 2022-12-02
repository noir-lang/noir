use hex::FromHexError;
use noirc_abi::{errors::InputParserError, input_parser::InputValue, AbiType};
use std::{fmt::Display, io::Write, path::PathBuf};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

#[derive(Debug)]
pub enum CliError {
    Generic(String),
    DestinationAlreadyExists(PathBuf),
    PathNotValid(PathBuf),
    ProofNotValid(FromHexError),
}

impl CliError {
    pub(crate) fn write(&self) -> ! {
        let mut stderr = StandardStream::stderr(ColorChoice::Always);
        stderr
            .set_color(ColorSpec::new().set_fg(Some(Color::Red)))
            .expect("cannot set color for stderr in StandardStream");
        writeln!(&mut stderr, "{}", self).expect("cannot write to stderr");

        std::process::exit(0)
    }
}

impl Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CliError::Generic(msg) => format!("Error: {}", msg),
                CliError::DestinationAlreadyExists(path) =>
                    format!("Error: destination {} already exists", path.display()),
                CliError::PathNotValid(path) => {
                    format!("Error: {} is not a valid path", path.display())
                }
                CliError::ProofNotValid(hex_error) => {
                    format!("Error: could not parse proof data ({})", hex_error)
                }
            }
        )
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

#[derive(Debug)]
pub enum AbiError {
    Generic(String),
    UnexpectedParams(Vec<String>),
    TypeMismatch { param_name: String, param_type: AbiType, value: InputValue },
    MissingParam(String),
    UndefinedInput(String),
}

impl Display for AbiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                AbiError::Generic(msg) => msg.clone(),
                AbiError::UnexpectedParams(unexpected_params) =>
                    format!("Received parameters not expected by ABI: {:?}", unexpected_params),
                AbiError::TypeMismatch { param_name, param_type, value } => {
                    format!(
                            "The parameter {} is expected to be a {:?} but found incompatible value {:?}",
                            param_name, param_type, value
                        )
                }
                AbiError::MissingParam(name) => {
                    format!("ABI expects the parameter `{}`, but this was not found", name)
                }
                AbiError::UndefinedInput(name) => {
                    format!("Input value `{}` is not defined", name)
                }
            }
        )
    }
}
