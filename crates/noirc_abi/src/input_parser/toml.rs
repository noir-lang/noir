use super::InputValue;
use crate::errors::InputParserError;
use acvm::FieldElement;
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

/// Converts the Toml mapping to the native representation that the compiler
/// understands for Inputs
fn toml_map_to_field(
    toml_map: BTreeMap<String, TomlTypes>,
) -> Result<BTreeMap<String, InputValue>, InputParserError> {
    let mut field_map = BTreeMap::new();

    for (parameter, value) in toml_map {
        match value {
            TomlTypes::String(string) => {
                let new_value = parse_str(&string)?;
                check_toml_map_duplicates(&mut field_map, parameter, InputValue::Field(new_value))?
            }
            TomlTypes::Integer(integer) => {
                let new_value = parse_str(&integer.to_string())?;
                check_toml_map_duplicates(&mut field_map, parameter, InputValue::Field(new_value))?
            }
            TomlTypes::ArrayNum(arr_num) => {
                let array_elements: Result<Vec<_>, _> =
                    arr_num.into_iter().map(|elem_num| parse_str(&elem_num.to_string())).collect();
                check_toml_map_duplicates(
                    &mut field_map,
                    parameter,
                    InputValue::Vec(array_elements?),
                )?
            }
            TomlTypes::ArrayString(arr_str) => {
                let array_elements: Result<Vec<_>, _> =
                    arr_str.into_iter().map(|elem_str| parse_str(&elem_str)).collect();
                check_toml_map_duplicates(
                    &mut field_map,
                    parameter,
                    InputValue::Vec(array_elements?),
                )?
            }
        }
    }

    Ok(field_map)
}

fn check_toml_map_duplicates(
    field_map: &mut BTreeMap<String, InputValue>,
    parameter: String,
    new_value: InputValue,
) -> Result<(), InputParserError> {
    match field_map.insert(parameter.clone(), new_value) {
        Some(_) => Err(InputParserError::DuplicateVariableName(parameter)),
        None => Ok(()),
    }
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

fn parse_str(value: &str) -> Result<FieldElement, InputParserError> {
    if value.starts_with("0x") {
        FieldElement::from_hex(value).ok_or_else(|| InputParserError::ParseHexStr(value.to_owned()))
    } else {
        let val: i128 = value
            .parse::<i128>()
            .map_err(|err_msg| InputParserError::ParseStr(err_msg.to_string()))?;
        Ok(FieldElement::from(val))
    }
}
