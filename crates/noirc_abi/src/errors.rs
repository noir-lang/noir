use crate::{input_parser::InputValue, AbiParameter, AbiType};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InputParserError {
    #[error("input.toml file is badly formed, could not parse, {0}")]
    ParseTomlMap(String),
    #[error("Expected witness values to be integers, provided value causes `{0}` error")]
    ParseStr(String),
    #[error("Could not parse hex value {0}")]
    ParseHexStr(String),
    #[error("duplicate variable name {0}")]
    DuplicateVariableName(String),
    #[error("cannot parse a string toml type into {0:?}")]
    AbiTypeMismatch(AbiType),
}

#[derive(Debug, Error)]
pub enum AbiError {
    #[error("{0}")]
    Generic(String),
    #[error("Received parameters not expected by ABI: {0:?}")]
    UnexpectedParams(Vec<String>),
    #[error("The parameter {} is expected to be a {:?} but found incompatible value {value:?}", .param.name, .param.typ)]
    TypeMismatch { param: AbiParameter, value: InputValue },
    #[error("ABI expects the parameter `{0}`, but this was not found")]
    MissingParam(String),
    #[error("Input value `{0}` is not defined")]
    UndefinedInput(String),
    #[error("ABI specifies an input of length {expected} but received input of length {actual}")]
    UnexpectedInputLength { expected: u32, actual: u32 },
}
