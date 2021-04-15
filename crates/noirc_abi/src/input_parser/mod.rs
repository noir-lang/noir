mod toml;

use std::{collections::BTreeMap, path::Path};

use noir_field::{Bn254Scalar, FieldElement};

use crate::AbiType;

/// This is what all formats eventually transform into
/// For example, a toml file will parse into TomlTypes
/// and those TomlTypes will be mapped to Value
#[derive(Debug, Clone)]
pub enum InputValue<F: FieldElement> {
    Field(F),
    Vec(Vec<F>),
}

impl<F: FieldElement> InputValue<F> {
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
        }
    }
}

/// Parses the initial Witness Values that are needed to seed the
/// Partial Witness generator
pub trait InitialWitnessParser<F: FieldElement> {
    fn parse_initial_witness<P: AsRef<Path>>(&self, path: P) -> BTreeMap<String, InputValue<F>>;
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
    ) -> BTreeMap<String, InputValue<Bn254Scalar>> {
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
