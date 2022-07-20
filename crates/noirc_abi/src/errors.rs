#[derive(Debug)]
pub enum InputParserError {
    MissingTomlFile(String),
    ParseTomlMap(String),
    ParseStr(String),
    DuplicateVariableName(String),
}

impl std::fmt::Display for InputParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
