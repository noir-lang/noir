mod json;
mod toml;

use std::collections::BTreeMap;

use acvm::FieldElement;
use serde::Serialize;

use crate::errors::{AbiError, InputParserError};
use crate::{decode_value, Abi, AbiType};
/// This is what all formats eventually transform into
/// For example, a toml file will parse into TomlTypes
/// and those TomlTypes will be mapped to Value
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum InputValue {
    Field(FieldElement),
    String(String),
    Vec(Vec<InputValue>),
    Struct(BTreeMap<String, InputValue>),
}

impl InputValue {
    /// Checks whether the ABI type matches the InputValue type
    /// and also their arity
    pub fn matches_abi(&self, abi_param: &AbiType) -> bool {
        match (self, abi_param) {
            (InputValue::Field(_), AbiType::Field) => true,
            (InputValue::Field(field_element), AbiType::Integer { width, .. }) => {
                field_element.num_bits() <= *width
            }
            (InputValue::Field(field_element), AbiType::Boolean) => {
                field_element.is_one() || field_element.is_zero()
            }

            (InputValue::Vec(array_elements), AbiType::Array { length, typ, .. }) => {
                if array_elements.len() != *length as usize {
                    return false;
                }
                // Check that all of the array's elements' values match the ABI as well.
                array_elements.iter().all(|input_value| input_value.matches_abi(typ))
            }

            (InputValue::String(string), AbiType::String { length }) => {
                string.len() == *length as usize
            }

            (InputValue::Struct(map), AbiType::Struct { fields, .. }) => {
                if map.len() != fields.len() {
                    return false;
                }

                let field_types = BTreeMap::from_iter(fields.iter().cloned());

                // Check that all of the struct's fields' values match the ABI as well.
                map.iter().all(|(field_name, field_value)| {
                    if let Some(field_type) = field_types.get(field_name) {
                        field_value.matches_abi(field_type)
                    } else {
                        false
                    }
                })
            }

            // All other InputValue-AbiType combinations are fundamentally incompatible.
            _ => false,
        }
    }
}

/// In order to display an `InputValue` we need an `AbiType` to accurately
/// convert the value into a human-readable format.
pub struct InputValueDisplay {
    input_value: InputValue,
    abi_type: AbiType,
}

impl InputValueDisplay {
    pub fn try_from_fields(
        field_iterator: &mut impl Iterator<Item = FieldElement>,
        abi_type: AbiType,
    ) -> Result<Self, AbiError> {
        let input_value = decode_value(field_iterator, &abi_type)?;
        Ok(InputValueDisplay { input_value, abi_type })
    }
}

impl std::fmt::Display for InputValueDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // From the docs: https://doc.rust-lang.org/std/fmt/struct.Error.html
        // This type does not support transmission of an error other than that an error
        // occurred. Any extra information must be arranged to be transmitted through
        // some other means.
        let json_value = json::JsonTypes::try_from_input_value(&self.input_value, &self.abi_type)
            .map_err(|_| std::fmt::Error)?;
        write!(f, "{}", serde_json::to_string(&json_value).map_err(|_| std::fmt::Error)?)
    }
}

/// The different formats that are supported when parsing
/// the initial witness values
#[cfg_attr(test, derive(strum_macros::EnumIter))]
pub enum Format {
    Json,
    Toml,
}

impl Format {
    pub fn ext(&self) -> &'static str {
        match self {
            Format::Json => "json",
            Format::Toml => "toml",
        }
    }
}

impl Format {
    pub fn parse(
        &self,
        input_string: &str,
        abi: &Abi,
    ) -> Result<BTreeMap<String, InputValue>, InputParserError> {
        match self {
            Format::Json => json::parse_json(input_string, abi),
            Format::Toml => toml::parse_toml(input_string, abi),
        }
    }

    pub fn serialize(
        &self,
        input_map: &BTreeMap<String, InputValue>,
        abi: &Abi,
    ) -> Result<String, InputParserError> {
        match self {
            Format::Json => json::serialize_to_json(input_map, abi),
            Format::Toml => toml::serialize_to_toml(input_map, abi),
        }
    }
}

#[cfg(test)]
mod serialization_tests {
    use std::collections::BTreeMap;

    use acvm::FieldElement;
    use strum::IntoEnumIterator;

    use crate::{
        input_parser::InputValue, Abi, AbiParameter, AbiType, AbiVisibility, Sign, MAIN_RETURN_NAME,
    };

    use super::Format;

    #[test]
    fn serialization_round_trip() {
        let abi = Abi {
            parameters: vec![
                AbiParameter {
                    name: "foo".into(),
                    typ: AbiType::Field,
                    visibility: AbiVisibility::Private,
                },
                AbiParameter {
                    name: "bar".into(),
                    typ: AbiType::Struct {
                        fields: vec![
                            ("field1".into(), AbiType::Integer { sign: Sign::Unsigned, width: 8 }),
                            (
                                "field2".into(),
                                AbiType::Array { length: 2, typ: Box::new(AbiType::Boolean) },
                            ),
                        ],
                    },
                    visibility: AbiVisibility::Private,
                },
            ],
            return_type: Some(AbiType::String { length: 5 }),
            // These two fields are unused when serializing/deserializing to file.
            param_witnesses: BTreeMap::new(),
            return_witnesses: Vec::new(),
        };

        let input_map: BTreeMap<String, InputValue> = BTreeMap::from([
            ("foo".into(), InputValue::Field(FieldElement::one())),
            (
                "bar".into(),
                InputValue::Struct(BTreeMap::from([
                    ("field1".into(), InputValue::Field(255u128.into())),
                    (
                        "field2".into(),
                        InputValue::Vec(vec![
                            InputValue::Field(true.into()),
                            InputValue::Field(false.into()),
                        ]),
                    ),
                ])),
            ),
            (MAIN_RETURN_NAME.into(), InputValue::String("hello".to_owned())),
        ]);

        for format in Format::iter() {
            let serialized_inputs = format.serialize(&input_map, &abi).unwrap();

            let reconstructed_input_map = format.parse(&serialized_inputs, &abi).unwrap();

            assert_eq!(input_map, reconstructed_input_map);
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

#[cfg(test)]
mod test {
    use super::parse_str_to_field;

    #[test]
    fn parse_empty_str_fails() {
        // Check that this fails appropriately rather than being treated as 0, etc.
        assert!(parse_str_to_field("").is_err());
    }
}
