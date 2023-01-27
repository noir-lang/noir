use super::InputValue;
use crate::{errors::InputParserError, Abi, AbiType};
use acvm::FieldElement;
use iter_extended::btree_map;
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

    // The toml map is stored in an ordered BTreeMap. As the keys are strings the map is in alphanumerical order.
    // When parsing the toml map we recursively go through each field to enable struct inputs.
    // To match this map with the correct abi type we reorganize our abi by parameter name in a BTreeMap, while the struct fields
    // in the abi are already stored in a BTreeMap.
    let abi_map = abi.to_btree_map();

    let toml_map = toml_map_to_field(data, abi_map)?;
    Ok(toml_map)
}

pub(crate) fn serialise_to_toml(
    w_map: &BTreeMap<String, InputValue>,
) -> Result<String, InputParserError> {
    // Toml requires that values be emitted before tables. Thus, we must reorder our map in case a TomlTypes::Table comes before any other values in the toml map
    // BTreeMap orders by key and we need the name of the input as our key, so we must split our maps in case a table type has a name that is alphanumerically less
    // than any other value type
    let (tables_map, to_map): (BTreeMap<String, TomlTypes>, BTreeMap<String, TomlTypes>) = w_map
        .iter()
        .map(|(key, value)| (key.to_owned(), TomlTypes::from(value.clone())))
        .partition(|(_, v)| matches!(v, TomlTypes::Table(_)));

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
    abi_map: BTreeMap<String, AbiType>,
) -> Result<BTreeMap<String, InputValue>, InputParserError> {
    let mut field_map = BTreeMap::new();
    for (parameter, value) in toml_map {
        let mapped_value = match value {
            TomlTypes::String(string) => {
                let param_type = abi_map.get(&parameter).unwrap();
                match param_type {
                    AbiType::String { .. } => InputValue::String(string),
                    AbiType::Field | AbiType::Integer { .. } => {
                        if string.is_empty() {
                            InputValue::Undefined
                        } else {
                            InputValue::Field(parse_str_to_field(&string)?)
                        }
                    }
                    _ => return Err(InputParserError::AbiTypeMismatch(param_type.clone())),
                }
            }
            TomlTypes::Integer(integer) => {
                let new_value = FieldElement::from(i128::from(integer));

                InputValue::Field(new_value)
            }
            TomlTypes::Bool(boolean) => {
                let new_value = if boolean { FieldElement::one() } else { FieldElement::zero() };

                InputValue::Field(new_value)
            }
            TomlTypes::ArrayNum(arr_num) => {
                let array_elements: Vec<_> = arr_num
                    .into_iter()
                    .map(|elem_num| FieldElement::from(i128::from(elem_num)))
                    .collect();

                InputValue::Vec(array_elements)
            }
            TomlTypes::ArrayString(arr_str) => {
                let array_elements: Vec<_> = arr_str
                    .into_iter()
                    .map(|elem_str| parse_str_to_field(&elem_str).unwrap())
                    .collect();

                InputValue::Vec(array_elements)
            }
            TomlTypes::Table(table) => {
                let param_type = abi_map.get(&parameter).unwrap();
                let fields = match param_type {
                    AbiType::Struct { fields } => fields.clone(),
                    _ => return Err(InputParserError::AbiTypeMismatch(param_type.clone())),
                };
                let native_table = toml_map_to_field(table, fields)?;

                InputValue::Struct(native_table)
            }
        };

        if field_map.insert(parameter.clone(), mapped_value).is_some() {
            return Err(InputParserError::DuplicateVariableName(parameter));
        };
    }

    Ok(field_map)
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

impl From<InputValue> for TomlTypes {
    fn from(value: InputValue) -> Self {
        match value {
            InputValue::Field(f) => {
                let f_str = format!("0x{}", f.to_hex());
                TomlTypes::String(f_str)
            }
            InputValue::Vec(v) => {
                let array = v.iter().map(|i| format!("0x{}", i.to_hex())).collect();
                TomlTypes::ArrayString(array)
            }
            InputValue::String(s) => TomlTypes::String(s),
            InputValue::Struct(map) => {
                let map_with_toml_types =
                    btree_map(map, |(key, value)| (key, TomlTypes::from(value)));
                TomlTypes::Table(map_with_toml_types)
            }
            InputValue::Undefined => unreachable!(),
        }
    }
}

fn parse_str_to_field(value: &str) -> Result<FieldElement, InputParserError> {
    if value.starts_with("0x") {
        FieldElement::from_hex(value).ok_or_else(|| InputParserError::ParseHexStr(value.to_owned()))
    } else {
        value
            .parse::<i128>()
            .map_err(|err_msg| InputParserError::ParseStr(err_msg.to_string()))
            .map(FieldElement::from)
    }
}
