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
    pub fn ext(&self) -> &'static str {
        match self {
            Format::Toml => "toml",
        }
    }
}

impl Format {
    pub fn parse<P: AsRef<Path>>(&self, path: P, file_name: &str) -> BTreeMap<String, InputValue> {
        match self {
            Format::Toml => {
                let mut dir_path = path.as_ref().to_path_buf();
                dir_path.push(file_name);
                dir_path.set_extension(self.ext());
                toml::parse(dir_path)
            }
        }
    }
}
