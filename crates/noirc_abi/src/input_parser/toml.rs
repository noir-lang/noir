use acvm::FieldElement;
use serde_derive::Deserialize;
use std::{collections::BTreeMap, path::Path};

use super::InputValue;

pub(crate) fn parse<P: AsRef<Path>>(path_to_toml: P) -> BTreeMap<String, InputValue> {
    let path_to_toml = path_to_toml.as_ref();
    assert!(
        path_to_toml.exists(),
        "cannot find input file at located {}",
        path_to_toml.display()
    );

    // Get input.toml file as a string
    let input_as_string = std::fs::read_to_string(path_to_toml).unwrap();

    // Parse input.toml into a BTreeMap, converting the argument to field elements
    let data: BTreeMap<String, TomlTypes> =
        toml::from_str(&input_as_string).expect("input.toml file is badly formed, could not parse");

    toml_map_to_field(data)
}

/// Converts the Toml mapping to the native representation that the compiler
/// understands for Inputs
fn toml_map_to_field(toml_map: BTreeMap<String, TomlTypes>) -> BTreeMap<String, InputValue> {
    let mut field_map = BTreeMap::new();

    for (parameter, value) in toml_map {
        match value {
            TomlTypes::String(string) => {
                let old_value =
                    field_map.insert(parameter.clone(), InputValue::Field(parse_str(&string)));
                assert!(old_value.is_none(), "duplicate variable name {}", parameter);
            }
            TomlTypes::Integer(integer) => {
                let old_value = field_map.insert(
                    parameter.clone(),
                    InputValue::Field(parse_str(&integer.to_string())),
                );
                assert!(old_value.is_none(), "duplicate variable name {}", parameter);
            }
            TomlTypes::ArrayNum(arr_num) => {
                let array_elements: Vec<_> = arr_num
                    .into_iter()
                    .map(|elem_num| parse_str(&elem_num.to_string()))
                    .collect();

                let old_value =
                    field_map.insert(parameter.clone(), InputValue::Vec(array_elements));
                assert!(old_value.is_none(), "duplicate variable name {}", parameter);
            }
            TomlTypes::ArrayString(arr_str) => {
                let array_elements: Vec<_> = arr_str
                    .into_iter()
                    .map(|elem_str| parse_str(&elem_str))
                    .collect();

                let old_value =
                    field_map.insert(parameter.clone(), InputValue::Vec(array_elements));
                assert!(old_value.is_none(), "duplicate variable name {}", parameter);
            }
        }
    }

    field_map
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TomlTypes {
    // This is most likely going to be a hex string
    // But it is possible to support UTF-8
    String(String),
    // Just a regular integer, that can fit in 128 bits
    Integer(u64),
    // Array of regular integers
    ArrayNum(Vec<u64>),
    // Array of hexadecimal integers
    ArrayString(Vec<String>),
}

fn parse_str(value: &str) -> FieldElement {
    if value.starts_with("0x") {
        FieldElement::from_hex(value)
            .unwrap_or_else(|| panic!("Could not parse hex value {}", value))
    } else {
        let val: i128 = value
            .parse()
            .expect("Expected witness values to be integers");
        FieldElement::from(val)
    }
}
