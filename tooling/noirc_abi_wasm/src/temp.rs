//! This module contains vendored code from `noirc_abi` for converting JSON to `InputValue`s.
//! This should be removed in time.

use acvm::FieldElement;
use iter_extended::{try_btree_map, try_vecmap};
use noirc_abi::{errors::InputParserError, input_parser::InputValue, AbiType};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub(super) enum JsonTypes {
    // This is most likely going to be a hex string
    // But it is possible to support UTF-8
    String(String),
    // Just a regular integer, that can fit in 64 bits.
    //
    // The JSON spec does not specify any limit on the size of integer number types,
    // however we restrict the allowable size. Values which do not fit in a u64 should be passed
    // as a string.
    Integer(u64),
    // Simple boolean flag
    Bool(bool),
    // Array of JsonTypes
    Array(Vec<JsonTypes>),
    // Struct of JsonTypes
    Table(BTreeMap<String, JsonTypes>),
}

impl JsonTypes {
    pub(super) fn try_from_input_value(
        value: &InputValue,
        abi_type: &AbiType,
    ) -> Result<JsonTypes, InputParserError> {
        let json_value = match (value, abi_type) {
            (InputValue::Field(f), AbiType::Field | AbiType::Integer { .. }) => {
                let f_str = format!("0x{}", f.to_hex());
                JsonTypes::String(f_str)
            }
            (InputValue::Field(f), AbiType::Boolean) => JsonTypes::Bool(f.is_one()),

            (InputValue::Vec(vector), AbiType::Array { typ, .. }) => {
                let array =
                    try_vecmap(vector, |value| JsonTypes::try_from_input_value(value, typ))?;
                JsonTypes::Array(array)
            }

            (InputValue::String(s), AbiType::String { .. }) => JsonTypes::String(s.to_string()),

            (InputValue::Struct(map), AbiType::Struct { fields, .. }) => {
                let map_with_json_types = try_btree_map(fields, |(key, field_type)| {
                    JsonTypes::try_from_input_value(&map[key], field_type)
                        .map(|json_value| (key.to_owned(), json_value))
                })?;
                JsonTypes::Table(map_with_json_types)
            }

            _ => return Err(InputParserError::AbiTypeMismatch(abi_type.clone())),
        };
        Ok(json_value)
    }
}

pub(super) fn input_value_from_json_type(
    value: JsonTypes,
    param_type: &AbiType,
    arg_name: &str,
) -> Result<InputValue, InputParserError> {
    let input_value = match (value, param_type) {
        (JsonTypes::String(string), AbiType::String { .. }) => InputValue::String(string),
        (
            JsonTypes::String(string),
            AbiType::Field | AbiType::Integer { .. } | AbiType::Boolean,
        ) => InputValue::Field(parse_str_to_field(&string)?),

        (
            JsonTypes::Integer(integer),
            AbiType::Field | AbiType::Integer { .. } | AbiType::Boolean,
        ) => {
            let new_value = FieldElement::from(i128::from(integer));

            InputValue::Field(new_value)
        }

        (JsonTypes::Bool(boolean), AbiType::Boolean) => InputValue::Field(boolean.into()),

        (JsonTypes::Array(array), AbiType::Array { typ, .. }) => {
            let array_elements =
                try_vecmap(array, |value| input_value_from_json_type(value, typ, arg_name))?;
            InputValue::Vec(array_elements)
        }

        (JsonTypes::Table(table), AbiType::Struct { fields, .. }) => {
            let native_table = try_btree_map(fields, |(field_name, abi_type)| {
                // Check that json contains a value for each field of the struct.
                let field_id = format!("{arg_name}.{field_name}");
                let value = table
                    .get(field_name)
                    .ok_or_else(|| InputParserError::MissingArgument(field_id.clone()))?;
                input_value_from_json_type(value.clone(), abi_type, &field_id)
                    .map(|input_value| (field_name.to_string(), input_value))
            })?;

            InputValue::Struct(native_table)
        }

        (_, _) => return Err(InputParserError::AbiTypeMismatch(param_type.clone())),
    };

    Ok(input_value)
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
