mod toml;

use std::{collections::BTreeMap, path::Path};

use acvm::FieldElement;
use serde::Serialize;

use crate::errors::InputParserError;
use crate::AbiType;
/// This is what all formats eventually transform into
/// For example, a toml file will parse into TomlTypes
/// and those TomlTypes will be mapped to Value
#[derive(Debug, Clone, Serialize)]
pub enum InputValue {
    Field(FieldElement),
    Vec(Vec<FieldElement>),
    Undefined,
}

impl InputValue {
    /// Checks whether the ABI type matches the InputValue type
    /// and also their arity
    pub fn matches_abi(&self, abi_param: AbiType) -> bool {
        match (self, abi_param) {
            (InputValue::Field(_), AbiType::Field(_)) => true,
            (InputValue::Field(_), AbiType::Array { .. }) => false,
            (InputValue::Field(_), AbiType::Integer { .. }) => true,
            (InputValue::Vec(_), AbiType::Field(_)) => false,
            (InputValue::Vec(x), AbiType::Array { length, .. }) => x.len() == length as usize,
            (InputValue::Vec(_), AbiType::Integer { .. }) => false,
            (InputValue::Undefined, _) => true,
        }
    }
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
    pub fn parse<P: AsRef<Path>>(
        &self,
        path: P,
        file_name: &str,
    ) -> Result<BTreeMap<String, InputValue>, InputParserError> {
        match self {
            Format::Toml => {
                let mut dir_path = path.as_ref().to_path_buf();
                dir_path.push(file_name);
                dir_path.set_extension(self.ext());
                toml::parse(dir_path)
            }
        }
    }

    pub fn serialise<P: AsRef<Path>>(
        &self,
        path: P,
        file_name: &str,
        w_map: &BTreeMap<String, InputValue>,
    ) -> Result<(), InputParserError> {
        match self {
            Format::Toml => {
                let mut dir_path = path.as_ref().to_path_buf();
                dir_path.push(file_name);
                dir_path.set_extension(self.ext());
                toml::serialise(dir_path, w_map)?;
            }
        }
        Ok(())
    }
}
