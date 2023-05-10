use super::{parse_str_to_field, InputValue};
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

    // Convert arguments to field elements.
    let mut parsed_inputs = try_btree_map(abi.to_btree_map(), |(arg_name, abi_type)| {
        // Check that toml contains a value for each argument in the ABI.
        let value = data
            .get(&arg_name)
            .ok_or_else(|| InputParserError::MissingArgument(arg_name.clone()))?;

        InputValue::try_from_toml(value.clone(), &abi_type, &arg_name)
            .map(|input_value| (arg_name, input_value))
    })?;

    // If the toml file also includes a return value then we parse it as well.
    // This isn't required as the prover calculates the return value itself.
    if let (Some(return_type), Some(toml_return_value)) =
        (&abi.return_type, data.get(MAIN_RETURN_NAME))
    {
        let return_value =
            InputValue::try_from_toml(toml_return_value.clone(), return_type, MAIN_RETURN_NAME)?;
        parsed_inputs.insert(MAIN_RETURN_NAME.to_owned(), return_value);
    }

    Ok(parsed_inputs)
}

pub(crate) fn serialize_to_toml(
    w_map: &BTreeMap<String, InputValue>,
) -> Result<String, InputParserError> {
    let to_map: BTreeMap<_, _> =
        w_map.iter().map(|(key, value)| (key, TomlTypes::from(value.clone()))).collect();

    let toml_string = toml::to_string(&to_map)?;

    Ok(toml_string)
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
enum TomlTypes {
    // This is most likely going to be a hex string
    // But it is possible to support UTF-8
    String(String),
    // Just a regular integer, that can fit in 64 bits
    // Note that the toml spec specifies that all numbers are represented as `i64`s.
    Integer(u64),
    // Simple boolean flag
    Bool(bool),
    // Array of regular integers
    ArrayNum(Vec<u64>),
    // Array of hexadecimal integers
    ArrayString(Vec<String>),
    // Array of booleans
    ArrayBool(Vec<bool>),
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
        arg_name: &str,
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
            TomlTypes::Bool(boolean) => InputValue::Field(boolean.into()),
            TomlTypes::ArrayNum(arr_num) => {
                let array_elements =
                    vecmap(arr_num, |elem_num| FieldElement::from(i128::from(elem_num)));

                InputValue::Vec(array_elements)
            }
            TomlTypes::ArrayString(arr_str) => {
                let array_elements = try_vecmap(arr_str, |elem_str| parse_str_to_field(&elem_str))?;

                InputValue::Vec(array_elements)
            }
            TomlTypes::ArrayBool(arr_bool) => {
                let array_elements = vecmap(arr_bool, FieldElement::from);

                InputValue::Vec(array_elements)
            }

            TomlTypes::Table(table) => match param_type {
                AbiType::Struct { fields } => {
                    let native_table = try_btree_map(fields, |(field_name, abi_type)| {
                        // Check that toml contains a value for each field of the struct.
                        let field_id = format!("{arg_name}.{field_name}");
                        let value = table
                            .get(field_name)
                            .ok_or_else(|| InputParserError::MissingArgument(field_id.clone()))?;
                        InputValue::try_from_toml(value.clone(), abi_type, &field_id)
                            .map(|input_value| (field_name.to_string(), input_value))
                    })?;

                    InputValue::Struct(native_table)
                }
                _ => return Err(InputParserError::AbiTypeMismatch(param_type.clone())),
            },
        };

        Ok(input_value)
    }
}
