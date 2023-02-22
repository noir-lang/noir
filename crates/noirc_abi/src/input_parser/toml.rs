use super::InputValue;
use crate::{errors::InputParserError, Abi, AbiType, MAIN_RETURN_NAME};
use acvm::FieldElement;
use iter_extended::{btree_map, try_btree_map, try_vecmap, vecmap};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub(crate) fn parse_toml(
    input_string: &str,
    abi: &Abi,
) -> Result<BTreeMap<String, InputValue>, InputParserError> {
    // Parse input.toml into a BTreeMap.
    let data: BTreeMap<String, TomlTypes> = toml::from_str(input_string)?;

    // The toml map is stored in an ordered BTreeMap. As the keys are strings the map is in alphanumerical order.
    // When parsing the toml map we recursively go through each field to enable struct inputs.
    // To match this map with the correct abi type we reorganize our abi by parameter name in a BTreeMap, while the struct fields
    // in the abi are already stored in a BTreeMap.
    let mut abi_map = abi.to_btree_map();
    if let Some(return_type) = &abi.return_type {
        abi_map.insert(MAIN_RETURN_NAME.to_owned(), return_type.to_owned());
    }

    // Convert arguments to field elements.
    try_btree_map(data, |(key, value)| {
        InputValue::try_from_toml(value, &abi_map[&key]).map(|input_value| (key, input_value))
    })
}

pub(crate) fn serialize_to_toml(
    w_map: &BTreeMap<String, InputValue>,
) -> Result<String, InputParserError> {
    // Toml requires that values be emitted before tables. Thus, we must reorder our map in case a TomlTypes::Table comes before any other values in the toml map
    // BTreeMap orders by key and we need the name of the input as our key, so we must split our maps in case a table type has a name that is alphanumerically less
    // than any other value type
    let (tables_map, to_map): (BTreeMap<String, TomlTypes>, BTreeMap<String, TomlTypes>) = w_map
        .iter()
        .map(|(key, value)| (key.to_owned(), TomlTypes::from(value.clone())))
        .partition(|(_, v)| matches!(v, TomlTypes::Table(_)));

    let mut toml_string = toml::to_string(&to_map)?;
    let toml_string_tables = toml::to_string(&tables_map)?;

    toml_string.push_str(&toml_string_tables);

    Ok(toml_string)
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
        }
    }
}

impl InputValue {
    fn try_from_toml(
        value: TomlTypes,
        param_type: &AbiType,
    ) -> Result<InputValue, InputParserError> {
        let input_value = match value {
            TomlTypes::String(string) => match param_type {
                AbiType::String { .. } => InputValue::String(string),
                AbiType::Field | AbiType::Integer { .. } | AbiType::Boolean => {
                    InputValue::Field(parse_str_to_field(&string)?)
                }

                AbiType::Array { .. } | AbiType::Struct { .. } => {
                    return Err(InputParserError::AbiTypeMismatch(param_type.clone()))
                }
            },
            TomlTypes::Integer(integer) => {
                let new_value = FieldElement::from(i128::from(integer));

                InputValue::Field(new_value)
            }
            TomlTypes::Bool(boolean) => {
                let new_value = if boolean { FieldElement::one() } else { FieldElement::zero() };

                InputValue::Field(new_value)
            }
            TomlTypes::ArrayNum(arr_num) => {
                let array_elements =
                    vecmap(arr_num, |elem_num| FieldElement::from(i128::from(elem_num)));

                InputValue::Vec(array_elements)
            }
            TomlTypes::ArrayString(arr_str) => {
                let array_elements = try_vecmap(arr_str, |elem_str| parse_str_to_field(&elem_str))?;

                InputValue::Vec(array_elements)
            }
            TomlTypes::Table(table) => {
                let fields = match param_type {
                    AbiType::Struct { fields } => fields,
                    _ => return Err(InputParserError::AbiTypeMismatch(param_type.clone())),
                };
                let native_table = try_btree_map(table, |(key, value)| {
                    InputValue::try_from_toml(value, &fields[&key])
                        .map(|input_value| (key, input_value))
                })?;

                InputValue::Struct(native_table)
            }
        };

        Ok(input_value)
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

#[cfg(test)]
mod test {
    use super::parse_str_to_field;

    #[test]
    fn parse_empty_str_fails() {
        // Check that this fails appropriately rather than being treated as 0, etc.
        assert!(parse_str_to_field("").is_err());
    }
}
