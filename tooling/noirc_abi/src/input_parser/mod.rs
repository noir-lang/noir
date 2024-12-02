use num_bigint::{BigInt, BigUint};
use num_traits::{Num, Zero};
use std::collections::{BTreeMap, HashSet};
use thiserror::Error;

use acvm::{AcirField, FieldElement};
use serde::Serialize;

use crate::errors::InputParserError;
use crate::{Abi, AbiType};

pub mod json;
mod toml;

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

#[derive(Debug, Error)]
pub enum InputTypecheckingError {
    #[error("Value {value:?} does not fall within range of allowable values for a {typ:?}")]
    OutsideOfValidRange { path: String, typ: AbiType, value: InputValue },
    #[error("Type {typ:?} is expected to have length {expected_length} but value {value:?} has length {actual_length}")]
    LengthMismatch {
        path: String,
        typ: AbiType,
        value: InputValue,
        expected_length: usize,
        actual_length: usize,
    },
    #[error("Could not find value for required field `{expected_field}`. Found values for fields {found_fields:?}")]
    MissingField { path: String, expected_field: String, found_fields: Vec<String> },
    #[error("Additional unexpected field was provided for type {typ:?}. Found field named `{extra_field}`")]
    UnexpectedField { path: String, typ: AbiType, extra_field: String },
    #[error("Type {typ:?} and value {value:?} do not match")]
    IncompatibleTypes { path: String, typ: AbiType, value: InputValue },
}

impl InputTypecheckingError {
    pub(crate) fn path(&self) -> &str {
        match self {
            InputTypecheckingError::OutsideOfValidRange { path, .. }
            | InputTypecheckingError::LengthMismatch { path, .. }
            | InputTypecheckingError::MissingField { path, .. }
            | InputTypecheckingError::UnexpectedField { path, .. }
            | InputTypecheckingError::IncompatibleTypes { path, .. } => path,
        }
    }
}

impl InputValue {
    /// Checks whether the ABI type matches the InputValue type
    pub(crate) fn find_type_mismatch(
        &self,
        abi_param: &AbiType,
        path: String,
    ) -> Result<(), InputTypecheckingError> {
        match (self, abi_param) {
            (InputValue::Field(_), AbiType::Field) => Ok(()),
            (InputValue::Field(field_element), AbiType::Integer { width, .. }) => {
                if field_element.num_bits() <= *width {
                    Ok(())
                } else {
                    Err(InputTypecheckingError::OutsideOfValidRange {
                        path,
                        typ: abi_param.clone(),
                        value: self.clone(),
                    })
                }
            }
            (InputValue::Field(field_element), AbiType::Boolean) => {
                if field_element.is_one() || field_element.is_zero() {
                    Ok(())
                } else {
                    Err(InputTypecheckingError::OutsideOfValidRange {
                        path,
                        typ: abi_param.clone(),
                        value: self.clone(),
                    })
                }
            }

            (InputValue::Vec(array_elements), AbiType::Array { length, typ, .. }) => {
                if array_elements.len() != *length as usize {
                    return Err(InputTypecheckingError::LengthMismatch {
                        path,
                        typ: abi_param.clone(),
                        value: self.clone(),
                        expected_length: *length as usize,
                        actual_length: array_elements.len(),
                    });
                }
                // Check that all of the array's elements' values match the ABI as well.
                for (i, element) in array_elements.iter().enumerate() {
                    let mut path = path.clone();
                    path.push_str(&format!("[{i}]"));

                    element.find_type_mismatch(typ, path)?;
                }
                Ok(())
            }

            (InputValue::String(string), AbiType::String { length }) => {
                if string.len() == *length as usize {
                    Ok(())
                } else {
                    Err(InputTypecheckingError::LengthMismatch {
                        path,
                        typ: abi_param.clone(),
                        value: self.clone(),
                        actual_length: string.len(),
                        expected_length: *length as usize,
                    })
                }
            }

            (InputValue::Struct(map), AbiType::Struct { fields, .. }) => {
                for (field_name, field_type) in fields {
                    if let Some(value) = map.get(field_name) {
                        let mut path = path.clone();
                        path.push_str(&format!(".{field_name}"));
                        value.find_type_mismatch(field_type, path)?;
                    } else {
                        return Err(InputTypecheckingError::MissingField {
                            path,
                            expected_field: field_name.to_string(),
                            found_fields: map.keys().cloned().collect(),
                        });
                    }
                }

                if map.len() > fields.len() {
                    let expected_fields: HashSet<String> =
                        fields.iter().map(|(field, _)| field.to_string()).collect();
                    let extra_field = map.keys().find(|&key| !expected_fields.contains(key)).cloned().expect("`map` is larger than the expected type's `fields` so it must contain an unexpected field");
                    return Err(InputTypecheckingError::UnexpectedField {
                        path,
                        typ: abi_param.clone(),
                        extra_field: extra_field.to_string(),
                    });
                }

                Ok(())
            }

            (InputValue::Vec(vec_elements), AbiType::Tuple { fields }) => {
                if vec_elements.len() != fields.len() {
                    return Err(InputTypecheckingError::LengthMismatch {
                        path,
                        typ: abi_param.clone(),
                        value: self.clone(),
                        actual_length: vec_elements.len(),
                        expected_length: fields.len(),
                    });
                }
                // Check that all of the array's elements' values match the ABI as well.
                for (i, (element, expected_typ)) in vec_elements.iter().zip(fields).enumerate() {
                    let mut path = path.clone();
                    path.push_str(&format!(".{i}"));
                    element.find_type_mismatch(expected_typ, path)?;
                }
                Ok(())
            }

            // All other InputValue-AbiType combinations are fundamentally incompatible.
            _ => Err(InputTypecheckingError::IncompatibleTypes {
                path,
                typ: abi_param.clone(),
                value: self.clone(),
            }),
        }
    }

    /// Checks whether the ABI type matches the InputValue type.
    pub fn matches_abi(&self, abi_param: &AbiType) -> bool {
        self.find_type_mismatch(abi_param, String::new()).is_ok()
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

    use acvm::{AcirField, FieldElement};
    use strum::IntoEnumIterator;

    use crate::{
        input_parser::InputValue, Abi, AbiParameter, AbiReturnType, AbiType, AbiVisibility, Sign,
        MAIN_RETURN_NAME,
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
                    name: "signed_example".into(),
                    typ: AbiType::Integer { sign: Sign::Signed, width: 8 },
                    visibility: AbiVisibility::Private,
                },
                AbiParameter {
                    name: "bar".into(),
                    typ: AbiType::Struct {
                        path: "MyStruct".into(),
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
            return_type: Some(AbiReturnType {
                abi_type: AbiType::String { length: 5 },
                visibility: AbiVisibility::Public,
            }),
            error_types: Default::default(),
        };

        let input_map: BTreeMap<String, InputValue> = BTreeMap::from([
            ("foo".into(), InputValue::Field(FieldElement::one())),
            ("signed_example".into(), InputValue::Field(FieldElement::from(240u128))),
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
    let big_num = if let Some(hex) = value.strip_prefix("0x") {
        BigUint::from_str_radix(hex, 16)
    } else {
        BigUint::from_str_radix(value, 10)
    };
    big_num.map_err(|err_msg| InputParserError::ParseStr(err_msg.to_string())).and_then(|bigint| {
        if bigint < FieldElement::modulus() {
            Ok(field_from_big_uint(bigint))
        } else {
            Err(InputParserError::ParseStr(format!(
                "Input exceeds field modulus. Values must fall within [0, {})",
                FieldElement::modulus(),
            )))
        }
    })
}

fn parse_str_to_signed(value: &str, width: u32) -> Result<FieldElement, InputParserError> {
    let big_num = if let Some(hex) = value.strip_prefix("-0x") {
        BigInt::from_str_radix(hex, 16).map(|value| -value)
    } else if let Some(hex) = value.strip_prefix("0x") {
        BigInt::from_str_radix(hex, 16)
    } else {
        BigInt::from_str_radix(value, 10)
    };

    big_num.map_err(|err_msg| InputParserError::ParseStr(err_msg.to_string())).and_then(|bigint| {
        let modulus: BigInt = FieldElement::modulus().into();
        let bigint = if bigint.sign() == num_bigint::Sign::Minus {
            BigInt::from(2).pow(width) + bigint
        } else {
            bigint
        };
        if bigint.is_zero() || (bigint.sign() == num_bigint::Sign::Plus && bigint < modulus) {
            Ok(field_from_big_int(bigint))
        } else {
            Err(InputParserError::ParseStr(format!(
                "Input exceeds field modulus. Values must fall within [0, {})",
                FieldElement::modulus(),
            )))
        }
    })
}

fn field_from_big_uint(bigint: BigUint) -> FieldElement {
    FieldElement::from_be_bytes_reduce(&bigint.to_bytes_be())
}

fn field_from_big_int(bigint: BigInt) -> FieldElement {
    match bigint.sign() {
        num_bigint::Sign::Minus => {
            unreachable!(
                "Unsupported negative value; it should only be called with a positive value"
            )
        }
        num_bigint::Sign::NoSign => FieldElement::zero(),
        num_bigint::Sign::Plus => FieldElement::from_be_bytes_reduce(&bigint.to_bytes_be().1),
    }
}

fn field_to_signed_hex(f: FieldElement, bit_size: u32) -> String {
    let f_u128 = f.to_u128();
    let max = 2_u128.pow(bit_size - 1) - 1;
    if f_u128 > max {
        let f = FieldElement::from(2_u128.pow(bit_size) - f_u128);
        format!("-0x{}", f.to_hex())
    } else {
        format!("0x{}", f.to_hex())
    }
}

#[cfg(test)]
mod test {
    use acvm::{AcirField, FieldElement};
    use num_bigint::BigUint;

    use super::{parse_str_to_field, parse_str_to_signed};

    fn big_uint_from_field(field: FieldElement) -> BigUint {
        BigUint::from_bytes_be(&field.to_be_bytes())
    }

    #[test]
    fn parse_empty_str_fails() {
        // Check that this fails appropriately rather than being treated as 0, etc.
        assert!(parse_str_to_field("").is_err());
    }

    #[test]
    fn parse_fields_from_strings() {
        let fields = vec![
            FieldElement::zero(),
            FieldElement::one(),
            FieldElement::from(u128::MAX) + FieldElement::one(),
            // Equivalent to `FieldElement::modulus() - 1`
            -FieldElement::one(),
        ];

        for field in fields {
            let hex_field = format!("0x{}", field.to_hex());
            let field_from_hex = parse_str_to_field(&hex_field).unwrap();
            assert_eq!(field_from_hex, field);

            let dec_field = big_uint_from_field(field).to_string();
            let field_from_dec = parse_str_to_field(&dec_field).unwrap();
            assert_eq!(field_from_dec, field);
        }
    }

    #[test]
    fn rejects_noncanonical_fields() {
        let noncanonical_field = FieldElement::modulus().to_string();
        assert!(parse_str_to_field(&noncanonical_field).is_err());
    }

    #[test]
    fn test_parse_str_to_signed() {
        let value = parse_str_to_signed("1", 8).unwrap();
        assert_eq!(value, FieldElement::from(1_u128));

        let value = parse_str_to_signed("-1", 8).unwrap();
        assert_eq!(value, FieldElement::from(255_u128));

        let value = parse_str_to_signed("-1", 16).unwrap();
        assert_eq!(value, FieldElement::from(65535_u128));
    }
}

#[cfg(test)]
mod arbitrary {
    use proptest::prelude::*;

    use crate::{AbiType, Sign};

    pub(super) fn arb_signed_integer_type_and_value() -> BoxedStrategy<(AbiType, i64)> {
        (2u32..=64)
            .prop_flat_map(|width| {
                let typ = Just(AbiType::Integer { width, sign: Sign::Signed });
                let value = if width == 64 {
                    // Avoid overflow
                    i64::MIN..i64::MAX
                } else {
                    -(2i64.pow(width - 1))..(2i64.pow(width - 1) - 1)
                };
                (typ, value)
            })
            .boxed()
    }
}
