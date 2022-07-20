use std::path::PathBuf;

#[derive(Debug)]
pub enum InputParserError {
    MissingTomlFile(PathBuf),
    ParseTomlMap(String),
    ParseStr(String),
    ParseHexStr(String),
    DuplicateVariableName(String),
}

impl std::fmt::Display for InputParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputParserError::MissingTomlFile(path) => write!(f, "cannot find input file located at {:?}, run nargo build to generate the missing Prover and/or Verifier toml files", path),
            InputParserError::ParseTomlMap(err_msg) => write!(f, "input.toml file is badly formed, could not parse, {}", err_msg),
            InputParserError::ParseStr(err_msg) => write!(f, "Expected witness values to be integers, provided value causes `{}` error", err_msg),
            InputParserError::ParseHexStr(err_msg) => write!(f, "Could not parse hex value {}", err_msg),
            InputParserError::DuplicateVariableName(err_msg) => write!(f, "duplicate variable name {}", err_msg)
        }
    }
}
