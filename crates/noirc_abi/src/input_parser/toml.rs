use super::InputValue;
use crate::{errors::InputParserError, Abi, AbiType};
use acvm::FieldElement;
use serde::Serialize;
use serde_derive::Deserialize;
use std::collections::BTreeMap;

pub(crate) fn parse_toml(
    input_string: &str,
    abi: Abi,
) -> Result<BTreeMap<String, InputValue>, InputParserError> {
    // Parse input.toml into a BTreeMap, converting the argument to field elements
    let data: BTreeMap<String, TomlTypes> = toml::from_str(input_string)
        .map_err(|err_msg| InputParserError::ParseTomlMap(err_msg.to_string()))?;

    let mut sorted_abi = abi.clone();
    sorted_abi.sort();
    let flat_abi_types = sorted_abi.flattened_param_types();

    let (_, toml_map) = toml_map_to_field(data, flat_abi_types, 0)?;
    Ok(toml_map)
}

pub(crate) fn serialise_to_toml(
    w_map: &BTreeMap<String, InputValue>,
) -> Result<String, InputParserError> {
    let to_map = toml_remap(w_map);

    // Toml requires that values be emitted before tables. Thus, we must reorder our map in case a TomlTypes::Table comes before any other values in the toml map
    // BTreeMap orders by key and we need the name of the input as our key, so we must split our maps in case a table type has a name that is alphanumerically less
    // than any other value type
    let mut tables_map = BTreeMap::new();
    let to_map: BTreeMap<String, TomlTypes> = to_map
        .into_iter()
        .filter(|(k, v)| {
            if matches!(v, TomlTypes::Table(_)) {
                tables_map.insert(k.clone(), v.clone());
                false
            } else {
                true
            }
        })
        .collect();

    let mut toml_string = toml::to_string(&to_map)
        .map_err(|err_msg| InputParserError::ParseTomlMap(err_msg.to_string()))?;

    let toml_string_tables = toml::to_string(&tables_map)
        .map_err(|err_msg| InputParserError::ParseTomlMap(err_msg.to_string()))?;

    toml_string.push_str(&toml_string_tables);

    Ok(toml_string)
}

/// Converts the Toml mapping to the native representation that the compiler
/// understands for Inputs
fn toml_map_to_field(
    toml_map: BTreeMap<String, TomlTypes>,
    flat_abi_types: Vec<AbiType>,
    initial_index: usize,
) -> Result<(usize, BTreeMap<String, InputValue>), InputParserError> {
    let mut field_map = BTreeMap::new();
    let mut abi_index = initial_index;
    for (parameter, value) in toml_map {
        let mapped_value = match value {
            TomlTypes::String(string) => {
                let param_type = flat_abi_types[abi_index].clone();
                abi_index += 1;
                match param_type {
                    AbiType::String { .. } => InputValue::String(string),
                    AbiType::Field | AbiType::Integer { .. } => {
                        let new_value = parse_str_to_field(&string)?;
                        if let Some(field_element) = new_value {
                            InputValue::Field(field_element)
                        } else {
                            InputValue::Undefined
                        }
                    }
                    _ => unreachable!("abi type specified does not match with TomlType::String"),
                }
            }
            TomlTypes::Integer(integer) => {
                let new_value = FieldElement::from(i128::from(integer));

                abi_index += 1;
                InputValue::Field(new_value)
            }
            TomlTypes::Bool(boolean) => {
                let new_value = if boolean { FieldElement::one() } else { FieldElement::zero() };

                abi_index += 1;
                InputValue::Field(new_value)
            }
            TomlTypes::ArrayNum(arr_num) => {
                let array_elements: Vec<_> = arr_num
                    .into_iter()
                    .map(|elem_num| FieldElement::from(i128::from(elem_num)))
                    .collect();

                abi_index += 1;
                InputValue::Vec(array_elements)
            }
            TomlTypes::ArrayString(arr_str) => {
                let array_elements: Vec<_> = arr_str
                    .into_iter()
                    .map(|elem_str| parse_str_to_field(&elem_str).unwrap().unwrap())
                    .collect();

                abi_index += 1;
                InputValue::Vec(array_elements)
            }
            TomlTypes::Table(table) => {
                let (new_index, native_table) =
                    toml_map_to_field(table, flat_abi_types.clone(), abi_index)?;
                abi_index = new_index;

                InputValue::Struct(native_table)
            }
        };

        if field_map.insert(parameter.clone(), mapped_value).is_some() {
            return Err(InputParserError::DuplicateVariableName(parameter));
        };
    }

    Ok((abi_index, field_map))
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
            InputValue::String(s) => TomlTypes::String(s.clone()),
            InputValue::Struct(map) => {
                let map_with_toml_types = toml_remap(map);
                TomlTypes::Table(map_with_toml_types)
            }
            InputValue::Undefined => unreachable!(),
        };
        toml_map.insert(parameter.clone(), mapped_value);
    }
    toml_map
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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
    // Struct of TomlTypes
    Table(BTreeMap<String, TomlTypes>),
}

fn parse_str_to_field(value: &str) -> Result<Option<FieldElement>, InputParserError> {
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
