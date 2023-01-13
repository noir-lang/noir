use crate::{input_parser::InputValue, AbiParameter};

#[derive(Debug)]
pub enum InputParserError {
    ParseTomlMap(String),
    ParseStr(String),
    ParseHexStr(String),
    DuplicateVariableName(String),
}

impl std::fmt::Display for InputParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputParserError::ParseTomlMap(err_msg) => {
                write!(f, "input.toml file is badly formed, could not parse, {err_msg}")
            }
            InputParserError::ParseStr(err_msg) => write!(
                f,
                "Expected witness values to be integers, provided value causes `{err_msg}` error"
            ),
            InputParserError::ParseHexStr(err_msg) => {
                write!(f, "Could not parse hex value {err_msg}")
            }
            InputParserError::DuplicateVariableName(err_msg) => {
                write!(f, "duplicate variable name {err_msg}")
            }
        }
    }
}

#[derive(Debug)]
pub enum AbiError {
    Generic(String),
    UnexpectedParams(Vec<String>),
    TypeMismatch { param: AbiParameter, value: InputValue },
    MissingParam(String),
    UndefinedInput(String),
    UnexpectedInputLength { expected: u32, actual: u32 },
}

impl std::fmt::Display for AbiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                AbiError::Generic(msg) => msg.clone(),
                AbiError::UnexpectedParams(unexpected_params) =>
                    format!("Received parameters not expected by ABI: {unexpected_params:?}"),
                AbiError::TypeMismatch { param, value } => {
                    format!(
                        "The parameter {} is expected to be a {:?} but found incompatible value {:?}",
                        param.name, param.typ, value
                    )
                }
                AbiError::MissingParam(name) => {
                    format!("ABI expects the parameter `{name}`, but this was not found")
                }
                AbiError::UndefinedInput(name) => {
                    format!("Input value `{name}` is not defined")
                }
                AbiError::UnexpectedInputLength { expected, actual } => {
                    format!(
                        "ABI specifies an input of length {expected} but received input of length {actual}"
                    )
                }
            }
        )
    }
}
