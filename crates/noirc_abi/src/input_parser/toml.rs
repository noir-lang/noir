use super::InputValue;
use crate::errors::InputParserError;
use acvm::FieldElement;
use serde::Serialize;
use serde_derive::Deserialize;
use std::{collections::BTreeMap, path::Path};

pub(crate) fn parse<P: AsRef<Path>>(
    path_to_toml: P,
) -> Result<BTreeMap<String, InputValue>, InputParserError> {
    let path_to_toml = path_to_toml.as_ref();
    if !path_to_toml.exists() {
        return Err(InputParserError::MissingTomlFile(path_to_toml.to_path_buf()));
    }
    // Get input.toml file as a string
    let input_as_string = std::fs::read_to_string(path_to_toml).unwrap();

    // Parse input.toml into a BTreeMap, converting the argument to field elements
    let data: BTreeMap<String, TomlTypes> = toml::from_str(&input_as_string)
        .map_err(|err_msg| InputParserError::ParseTomlMap(err_msg.to_string()))?;
    toml_map_to_field(data)
}

pub fn serialise<P: AsRef<Path>>(
    path_to_toml: P,
    w_map: &BTreeMap<String, InputValue>,
) -> Result<(), InputParserError> {
    let to_map = toml_remap(w_map);
    let toml_string = toml::to_string(&to_map)
        .map_err(|err_msg| InputParserError::ParseTomlMap(err_msg.to_string()))?;
    std::fs::write(path_to_toml.as_ref(), toml_string).map_err(InputParserError::SaveTomlFile)?;
    Ok(())
}

/// Converts the Toml mapping to the native representation that the compiler
/// understands for Inputs
fn toml_map_to_field(
    toml_map: BTreeMap<String, TomlTypes>,
) -> Result<BTreeMap<String, InputValue>, InputParserError> {
    let mut field_map = BTreeMap::new();
    for (parameter, value) in toml_map {
        let mapped_value = match value {
            TomlTypes::String(string) => {
                let new_value = parse_str(&string)?;

                if let Some(field_element) = new_value {
                    InputValue::Field(field_element)
                } else {
                    InputValue::Undefined
                }
            }
            TomlTypes::Integer(integer) => {
                let new_value = parse_str(&integer.to_string())?;

                InputValue::Field(new_value.unwrap())
            }
            TomlTypes::Bool(boolean) => {
                let new_value = if boolean { FieldElement::one() } else { FieldElement::zero() };

                InputValue::Field(new_value)
            }
            TomlTypes::ArrayNum(arr_num) => {
                let array_elements: Vec<_> = arr_num
                    .into_iter()
                    .map(|elem_num| parse_str(&elem_num.to_string()).unwrap().unwrap())
                    .collect();

                InputValue::Vec(array_elements)
            }
            TomlTypes::ArrayString(arr_str) => {
                let array_elements: Vec<_> = arr_str
                    .into_iter()
                    .map(|elem_str| parse_str(&elem_str).unwrap().unwrap())
                    .collect();

                InputValue::Vec(array_elements)
            }
        };

        if field_map.insert(parameter.clone(), mapped_value).is_some() {
            return Err(InputParserError::DuplicateVariableName(parameter));
        };
    }

    Ok(field_map)
}

fn toml_remap(map: &BTreeMap<String, InputValue>) -> BTreeMap<String, TomlTypes> {
    let mut toml_map = BTreeMap::new();
    for (parameter, value) in map {
        let mapped_value = match value {
            InputValue::Field(f) => {
                let f_str = format!("0x{}", f.to_hex());
                TomlTypes::String(f_str)
            }
            InputValue::Vec(v) => {
                let array = v.iter().map(|i| format!("0x{}", i.to_hex())).collect();
                TomlTypes::ArrayString(array)
            }
            InputValue::Undefined => unreachable!(),
        };
        toml_map.insert(parameter.clone(), mapped_value);
    }
    toml_map
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum TomlTypes {
    // This is most likely going to be a hex string
    // But it is possible to support UTF-8
    String(String),
    // Just a regular integer, that can fit in 128 bits
    Integer(u64),
    // Simple boolean flag
    Bool(bool),
    // Array of regular integers
    ArrayNum(Vec<u64>),
    // Array of hexadecimal integers
    ArrayString(Vec<String>),
}

fn parse_str(value: &str) -> Result<Option<FieldElement>, InputParserError> {
    if value.is_empty() {
        Ok(None)
    } else if value.starts_with("0x") {
        let result = FieldElement::from_hex(value);
        if result.is_some() {
            Ok(result)
        } else {
            Err(InputParserError::ParseHexStr(value.to_owned()))
        }
    } else {
        let val: i128 = value
            .parse::<i128>()
            .map_err(|err_msg| InputParserError::ParseStr(err_msg.to_string()))?;
        Ok(Some(FieldElement::from(val)))
    }
}
