use super::{parse_str_to_field, InputValue};
use crate::{errors::InputParserError, Abi, AbiType, MAIN_RETURN_NAME};
use acvm::FieldElement;
use iter_extended::{try_btree_map, try_vecmap, vecmap};
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
    input_map: &BTreeMap<String, InputValue>,
    abi: &Abi,
) -> Result<String, InputParserError> {
    let mut toml_map = try_btree_map(abi.to_btree_map(), |(key, param_type)| {
        TomlTypes::try_from_input_value(&input_map[&key], &param_type)
            .map(|toml_value| (key.clone(), toml_value))
    })?;

    if let (Some(return_type), Some(return_value)) =
        (&abi.return_type, input_map.get(MAIN_RETURN_NAME))
    {
        let return_value = TomlTypes::try_from_input_value(return_value, return_type)?;
        toml_map.insert(MAIN_RETURN_NAME.to_owned(), return_value);
    }

    let toml_string = toml::to_string(&toml_map)?;

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

impl TomlTypes {
    fn try_from_input_value(
        value: &InputValue,
        abi_type: &AbiType,
    ) -> Result<TomlTypes, InputParserError> {
        let toml_value = match (value, abi_type) {
            (InputValue::Field(f), AbiType::Field | AbiType::Integer { .. }) => {
                let f_str = format!("0x{}", f.to_hex());
                TomlTypes::String(f_str)
            }
            (InputValue::Field(f), AbiType::Boolean) => TomlTypes::Bool(f.is_one()),

            (InputValue::Vec(v), AbiType::Array { typ, .. }) => match typ.as_ref() {
                AbiType::Field | AbiType::Integer { .. } => {
                    let array = v.iter().map(|i| format!("0x{}", i.to_hex())).collect();
                    TomlTypes::ArrayString(array)
                }
                AbiType::Boolean => {
                    let array = v.iter().map(|i| i.is_one()).collect();
                    TomlTypes::ArrayBool(array)
                }
                _ => return Err(InputParserError::AbiTypeMismatch(abi_type.clone())),
            },

            (InputValue::String(s), AbiType::String { .. }) => TomlTypes::String(s.to_string()),

            (InputValue::Struct(map), AbiType::Struct { fields }) => {
                let map_with_toml_types = try_btree_map(fields, |(key, field_type)| {
                    TomlTypes::try_from_input_value(&map[key], field_type)
                        .map(|toml_value| (key.to_owned(), toml_value))
                })?;
                TomlTypes::Table(map_with_toml_types)
            }

            _ => return Err(InputParserError::AbiTypeMismatch(abi_type.clone())),
        };
        Ok(toml_value)
    }
}

impl InputValue {
    fn try_from_toml(
        value: TomlTypes,
        param_type: &AbiType,
        arg_name: &str,
    ) -> Result<InputValue, InputParserError> {
        let input_value = match (value, param_type) {
            (TomlTypes::String(string), AbiType::String { .. }) => InputValue::String(string),
            (
                TomlTypes::String(string),
                AbiType::Field | AbiType::Integer { .. } | AbiType::Boolean,
            ) => InputValue::Field(parse_str_to_field(&string)?),

            (
                TomlTypes::Integer(integer),
                AbiType::Field | AbiType::Integer { .. } | AbiType::Boolean,
            ) => {
                let new_value = FieldElement::from(i128::from(integer));

                InputValue::Field(new_value)
            }

            (TomlTypes::Bool(boolean), AbiType::Boolean) => InputValue::Field(boolean.into()),

            (TomlTypes::ArrayNum(arr_num), AbiType::Array { typ, .. })
                if matches!(
                    typ.as_ref(),
                    AbiType::Field | AbiType::Integer { .. } | AbiType::Boolean
                ) =>
            {
                let array_elements =
                    vecmap(arr_num, |elem_num| FieldElement::from(i128::from(elem_num)));

                InputValue::Vec(array_elements)
            }
            (TomlTypes::ArrayString(arr_str), AbiType::Array { typ, .. })
                if matches!(
                    typ.as_ref(),
                    AbiType::Field | AbiType::Integer { .. } | AbiType::Boolean
                ) =>
            {
                let array_elements = try_vecmap(arr_str, |elem_str| parse_str_to_field(&elem_str))?;

                InputValue::Vec(array_elements)
            }
            (TomlTypes::ArrayBool(arr_bool), AbiType::Array { typ, .. })
                if matches!(typ.as_ref(), AbiType::Boolean) =>
            {
                let array_elements = vecmap(arr_bool, FieldElement::from);

                InputValue::Vec(array_elements)
            }

            (TomlTypes::Table(table), AbiType::Struct { fields }) => {
                let native_table = try_btree_map(fields, |(field_name, abi_type)| {
                    // Check that json contains a value for each field of the struct.
                    let field_id = format!("{arg_name}.{field_name}");
                    let value = table
                        .get(field_name)
                        .ok_or_else(|| InputParserError::MissingArgument(field_id.clone()))?;
                    InputValue::try_from_toml(value.clone(), abi_type, &field_id)
                        .map(|input_value| (field_name.to_string(), input_value))
                })?;

                InputValue::Struct(native_table)
            }

            (_, _) => return Err(InputParserError::AbiTypeMismatch(param_type.clone())),
        };

        Ok(input_value)
    }
}
