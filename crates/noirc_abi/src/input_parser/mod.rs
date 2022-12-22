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
            (InputValue::Field(_), AbiType::Field) => true,
            (InputValue::Field(_), AbiType::Array { .. }) => false,
            (InputValue::Field(_), AbiType::Integer { .. }) => true,
            (InputValue::Field(_), AbiType::Struct { .. }) => false,

            (InputValue::Vec(_), AbiType::Field) => false,
            (InputValue::Vec(x), AbiType::Array { length, .. }) => x.len() == *length as usize,
            (InputValue::Vec(_), AbiType::Integer { .. }) => false,
            (InputValue::Vec(_), AbiType::Struct { .. }) => false,

            (InputValue::Struct(_), AbiType::Field) => false,
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
    pub fn parse(
        &self,
        input_string: &str,
    ) -> Result<BTreeMap<String, InputValue>, InputParserError> {
        match self {
            Format::Toml => toml::parse_toml(input_string),
        }
    }

    pub fn serialise(
        &self,
        w_map: &BTreeMap<String, InputValue>,
    ) -> Result<String, InputParserError> {
        match self {
            Format::Toml => toml::serialise_to_toml(w_map),
        }
    }
}
