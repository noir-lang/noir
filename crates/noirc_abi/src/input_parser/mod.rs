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
    Struct(BTreeMap<String, InputValue>),
    Undefined,
}

impl InputValue {
    /// Checks whether the ABI type matches the InputValue type
    /// and also their arity
    pub fn matches_abi(&self, abi_param: &AbiType) -> bool {
        match (self, abi_param) {
            (InputValue::Field(_), AbiType::Field(_)) => true,
            (InputValue::Field(_), AbiType::Array { .. }) => false,
            (InputValue::Field(_), AbiType::Integer { .. }) => true,
            (InputValue::Field(_), AbiType::Struct { .. }) => false,

            (InputValue::Vec(_), AbiType::Field(_)) => false,
            (InputValue::Vec(x), AbiType::Array { length, .. }) => x.len() == *length as usize,
            (InputValue::Vec(_), AbiType::Integer { .. }) => false,
            (InputValue::Vec(_), AbiType::Struct { .. }) => false,

            (InputValue::Struct(_), AbiType::Field(_)) => false,
            (InputValue::Struct(_), AbiType::Array { .. }) => false,
            (InputValue::Struct(_), AbiType::Integer { .. }) => false,
            (InputValue::Struct(map), AbiType::Struct { fields, .. }) => map.len() == fields.len(),

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
        let input_string = self.load_file(path, file_name)?;
        self.parse_string(&input_string)
    }

    pub fn serialise<P: AsRef<Path>>(
        &self,
        path: P,
        file_name: &str,
        w_map: &BTreeMap<String, InputValue>,
    ) -> Result<(), InputParserError> {
        let input_string = self.serialise_to_string(w_map)?;
        self.write_file(path, file_name, input_string)
    }

    fn load_file<P: AsRef<Path>>(
        &self,
        path: P,
        file_name: &str,
    ) -> Result<String, InputParserError> {
        let file_path = {
            let mut dir_path = path.as_ref().to_path_buf();
            dir_path.push(file_name);
            dir_path.set_extension(self.ext());
            dir_path
        };

        if !file_path.exists() {
            return Err(InputParserError::MissingTomlFile(file_path.to_path_buf()));
        }

        // Get input as a string
        let input_as_string = std::fs::read_to_string(file_path).unwrap();
        Ok(input_as_string)
    }

    fn write_file<P: AsRef<Path>>(
        &self,
        path: P,
        file_name: &str,
        file_contents: String,
    ) -> Result<(), InputParserError> {
        let file_path = {
            let mut dir_path = path.as_ref().to_path_buf();
            dir_path.push(file_name);
            dir_path.set_extension(self.ext());
            dir_path
        };
        std::fs::write(file_path, file_contents).map_err(InputParserError::SaveTomlFile)?;
        Ok(())
    }

    fn parse_string(
        &self,
        input_string: &str,
    ) -> Result<BTreeMap<String, InputValue>, InputParserError> {
        match self {
            Format::Toml => toml::parse_toml(input_string),
        }
    }

    fn serialise_to_string(
        &self,
        w_map: &BTreeMap<String, InputValue>,
    ) -> Result<String, InputParserError> {
        match self {
            Format::Toml => toml::serialise_to_toml(w_map),
        }
    }
}
