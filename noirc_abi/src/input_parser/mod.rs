mod toml;

use std::{collections::BTreeMap, path::Path};

use noir_field::FieldElement;

/// This is what all formats eventually transform into
/// For example, a toml file will parse into TomlTypes
/// and those TomlTypes will be mapped to Value
#[derive(Debug, Clone)]
pub enum InputValue {
    Field(FieldElement),
    Vec(Vec<FieldElement>),
}

/// Parses the initial Witness Values that are needed to seed the
/// Partial Witness generator
pub trait InitialWitnessParser {
    fn parse_initial_witness<P: AsRef<Path>>(&self, path: P) -> BTreeMap<String, InputValue>;
}

/// The different formats that are supported when parsing
/// the initial witness values
pub enum Format {
    Toml,
}

impl Format {
    /// The name of the file that this Formatter can parse
    /// Eg. The Toml parser will look for an `input.toml`
    /// file in the directory.
    pub fn file_name(&self) -> &'static str {
        match self {
            Format::Toml => "input.toml",
        }
    }
}

impl Format {
    pub fn parse<P: AsRef<Path>>(&self, path: P) -> BTreeMap<String, InputValue> {
        match self {
            Format::Toml => {
                let mut dir_path = path.as_ref().to_path_buf();
                dir_path.push(self.file_name());
                toml::parse(dir_path)
            }
        }
    }
}
