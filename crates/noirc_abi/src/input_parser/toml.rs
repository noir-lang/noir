use acvm::FieldElement;
use serde_derive::Deserialize;
use std::{collections::BTreeMap, path::Path};

use super::InputValue;

pub(crate) fn parse<P: AsRef<Path>>(
    path_to_toml: P,
) -> Result<BTreeMap<String, InputValue>, String> {
    let path_to_toml = path_to_toml.as_ref();
    if !path_to_toml.exists() {
        return Err(format!("cannot find input file at located {}, run nargo build to generate the missing Prover and/or Verifier toml files", path_to_toml.display()));
    }

    // Get input.toml file as a string
    let input_as_string = std::fs::read_to_string(path_to_toml).unwrap();

    // Parse input.toml into a BTreeMap, converting the argument to field elements
    let data: BTreeMap<String, TomlTypes> =
        toml::from_str(&input_as_string).expect("input.toml file is badly formed, could not parse");

    toml_map_to_field(data)
}

/// Converts the Toml mapping to the native representation that the compiler
/// understands for Inputs
fn toml_map_to_field(
    toml_map: BTreeMap<String, TomlTypes>,
) -> Result<BTreeMap<String, InputValue>, String> {
    let mut field_map = BTreeMap::new();

    for (parameter, value) in toml_map {
        match value {
            TomlTypes::String(string) => {
                let new_value = parse_str(&string)?;
                let old_value = field_map.insert(parameter.clone(), InputValue::Field(new_value));
                if !old_value.is_none() {
                    return Err(format!("duplicate variable name {}", parameter));
                }
            }
            TomlTypes::Integer(integer) => {
                let new_value = parse_str(&integer.to_string())?;
                let old_value = field_map.insert(parameter.clone(), InputValue::Field(new_value));
                if !old_value.is_none() {
                    return Err(format!("duplicate variable name {}", parameter));
                }
            }
            TomlTypes::ArrayNum(arr_num) => {
                let array_elements: Result<Vec<_>, _> =
                    arr_num.into_iter().map(|elem_num| parse_str(&elem_num.to_string())).collect();
                let old_value =
                    field_map.insert(parameter.clone(), InputValue::Vec(array_elements?));
                if !old_value.is_none() {
                    return Err(format!("duplicate variable name {}", parameter));
                }
            }
            TomlTypes::ArrayString(arr_str) => {
                let array_elements: Result<Vec<_>, _> =
                    arr_str.into_iter().map(|elem_str| parse_str(&elem_str)).collect();
                let old_value =
                    field_map.insert(parameter.clone(), InputValue::Vec(array_elements?));
                if !old_value.is_none() {
                    return Err(format!("duplicate variable name {}", parameter));
                }
            }
        }
    }

    Ok(field_map)
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

fn parse_str(value: &str) -> Result<FieldElement, String> {
    if value.starts_with("0x") {
        match FieldElement::from_hex(value) {
            None => Err(format!("Could not parse hex value {}", value)),
            Some(val) => Ok(val),
        }
    } else {
        let val: i128 = match value.parse() {
            Err(msg) => {
                return Err(format!(
                    "Expected witness values to be integers, provided value `{}` causes `{}` error",
                    value, msg
                ))
            }
            Ok(parsed_val) => parsed_val,
        };
        Ok(FieldElement::from(val))
    }
}
