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
                write!(f, "input.toml file is badly formed, could not parse, {}", err_msg)
            }
            InputParserError::ParseStr(err_msg) => write!(
                f,
                "Expected witness values to be integers, provided value causes `{}` error",
                err_msg
            ),
            InputParserError::ParseHexStr(err_msg) => {
                write!(f, "Could not parse hex value {}", err_msg)
            }
            InputParserError::DuplicateVariableName(err_msg) => {
                write!(f, "duplicate variable name {}", err_msg)
            }
        }
    }
}
