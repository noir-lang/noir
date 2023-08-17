use super::PrintableValue;
use crate::{InputParserError, PrintableType};
use acvm::FieldElement;
use iter_extended::{try_btree_map, try_vecmap};
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
        value: &PrintableValue,
        abi_type: &PrintableType,
    ) -> Result<JsonTypes, InputParserError> {
        let json_value = match (value, abi_type) {
            (PrintableValue::Field(f), PrintableType::Field | PrintableType::Integer { .. }) => {
                JsonTypes::String(Self::format_field_string(*f))
            }
            (PrintableValue::Field(f), PrintableType::Boolean) => JsonTypes::Bool(f.is_one()),

            (PrintableValue::Vec(vector), PrintableType::Array { typ, .. }) => {
                let array =
                    try_vecmap(vector, |value| JsonTypes::try_from_input_value(value, typ))?;
                JsonTypes::Array(array)
            }

            (PrintableValue::String(s), PrintableType::String { .. }) => {
                JsonTypes::String(s.to_string())
            }

            (PrintableValue::Struct(map), PrintableType::Struct { fields, .. }) => {
                let map_with_json_types = try_btree_map(fields, |(key, field_type)| {
                    JsonTypes::try_from_input_value(&map[key], field_type)
                        .map(|json_value| (key.to_owned(), json_value))
                })?;
                JsonTypes::Table(map_with_json_types)
            }

            _ => return Err(InputParserError::PrintableTypeMismatch(abi_type.clone())),
        };
        Ok(json_value)
    }

    /// This trims any leading zeroes.
    /// A singular '0' will be prepended as well if the trimmed string has an odd length.
    /// A hex string's length needs to be even to decode into bytes, as two digits correspond to
    /// one byte.
    fn format_field_string(field: FieldElement) -> String {
        if field.is_zero() {
            return "0x00".to_owned();
        }
        let mut trimmed_field = field.to_hex().trim_start_matches('0').to_owned();
        if trimmed_field.len() % 2 != 0 {
            trimmed_field = "0".to_owned() + &trimmed_field;
        }
        "0x".to_owned() + &trimmed_field
    }
}

impl std::fmt::Display for JsonTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // From the docs: https://doc.rust-lang.org/std/fmt/struct.Error.html
        // This type does not support transmission of an error other than that an error
        // occurred. Any extra information must be arranged to be transmitted through
        // some other means.
        write!(f, "{}", serde_json::to_string(&self).map_err(|_| std::fmt::Error)?)
    }
}
