use std::{collections::BTreeMap, str};

use acvm::FieldElement;
use iter_extended::vecmap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

mod json;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sign {
    Unsigned,
    Signed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum PrintableType {
    Field,
    Array {
        length: u64,
        #[serde(rename = "type")]
        typ: Box<PrintableType>,
    },
    Integer {
        sign: Sign,
        width: u32,
    },
    Boolean,
    Struct {
        name: String,
        fields: Vec<(String, PrintableType)>,
    },
    String {
        length: u64,
    },
}

impl PrintableType {
    /// Returns the number of field elements required to represent the type once encoded.
    pub fn field_count(&self) -> u32 {
        match self {
            Self::Field | Self::Integer { .. } | Self::Boolean => 1,
            Self::Array { length, typ } => typ.field_count() * (*length as u32),
            Self::Struct { fields, .. } => {
                fields.iter().fold(0, |acc, (_, field_type)| acc + field_type.field_count())
            }
            Self::String { length } => *length as u32,
        }
    }
}

/// This is what all formats eventually transform into
/// For example, a toml file will parse into TomlTypes
/// and those TomlTypes will be mapped to Value
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum PrintableValue {
    Field(FieldElement),
    String(String),
    Vec(Vec<PrintableValue>),
    Struct(BTreeMap<String, PrintableValue>),
}

/// In order to display an `PrintableValue` we need an `PrintableType` to accurately
/// convert the value into a human-readable format.
pub struct PrintableValueDisplay {
    input_value: PrintableValue,
    typ: PrintableType,
}

impl PrintableValueDisplay {
    pub fn try_from_fields(
        field_iterator: &mut impl Iterator<Item = FieldElement>,
        typ: PrintableType,
    ) -> Result<Self, AbiError> {
        let input_value = decode_value(field_iterator, &typ)?;
        Ok(PrintableValueDisplay { input_value, typ })
    }
}

impl std::fmt::Display for PrintableValueDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // From the docs: https://doc.rust-lang.org/std/fmt/struct.Error.html
        // This type does not support transmission of an error other than that an error
        // occurred. Any extra information must be arranged to be transmitted through
        // some other means.
        let json_value = json::JsonTypes::try_from_input_value(&self.input_value, &self.typ)
            .map_err(|_| std::fmt::Error)?;
        write!(f, "{}", serde_json::to_string(&json_value).map_err(|_| std::fmt::Error)?)
    }
}

fn decode_value(
    field_iterator: &mut impl Iterator<Item = FieldElement>,
    value_type: &PrintableType,
) -> Result<PrintableValue, AbiError> {
    // This function assumes that `field_iterator` contains enough `FieldElement`s in order to decode a `value_type`
    // `Abi.decode` enforces that the encoded inputs matches the expected length defined by the ABI so this is safe.
    let value = match value_type {
        PrintableType::Field | PrintableType::Integer { .. } | PrintableType::Boolean => {
            let field_element = field_iterator.next().unwrap();

            PrintableValue::Field(field_element)
        }
        PrintableType::Array { length, typ } => {
            let length = *length as usize;
            let mut array_elements = Vec::with_capacity(length);
            for _ in 0..length {
                array_elements.push(decode_value(field_iterator, typ)?);
            }

            PrintableValue::Vec(array_elements)
        }
        PrintableType::String { length } => {
            let field_elements: Vec<FieldElement> = field_iterator.take(*length as usize).collect();

            PrintableValue::String(decode_string_value(&field_elements))
        }
        PrintableType::Struct { fields, .. } => {
            let mut struct_map = BTreeMap::new();

            for (field_key, param_type) in fields {
                let field_value = decode_value(field_iterator, param_type)?;

                struct_map.insert(field_key.to_owned(), field_value);
            }

            PrintableValue::Struct(struct_map)
        }
    };

    Ok(value)
}

pub fn decode_string_value(field_elements: &[FieldElement]) -> String {
    let string_as_slice = vecmap(field_elements, |e| {
        let mut field_as_bytes = e.to_be_bytes();
        let char_byte = field_as_bytes.pop().unwrap(); // A character in a string is represented by a u8, thus we just want the last byte of the element
        assert!(field_as_bytes.into_iter().all(|b| b == 0)); // Assert that the rest of the field element's bytes are empty
        char_byte
    });

    let final_string = str::from_utf8(&string_as_slice).unwrap();
    final_string.to_owned()
}

#[derive(Debug, Error)]
pub enum InputParserError {
    #[error("input file is badly formed, could not parse, {0}")]
    ParseInputMap(String),
    #[error("Expected witness values to be integers, provided value causes `{0}` error")]
    ParseStr(String),
    #[error("Could not parse hex value {0}")]
    ParseHexStr(String),
    #[error("duplicate variable name {0}")]
    DuplicateVariableName(String),
    #[error("cannot parse value into {0:?}")]
    PrintableTypeMismatch(PrintableType),
    #[error("Expected argument `{0}`, but none was found")]
    MissingArgument(String),
}

impl From<serde_json::Error> for InputParserError {
    fn from(err: serde_json::Error) -> Self {
        Self::ParseInputMap(err.to_string())
    }
}

#[derive(Debug, Error)]
pub enum AbiError {
    #[error("{0}")]
    Generic(String),
    #[error("Received parameters not expected by ABI: {0:?}")]
    UnexpectedParams(Vec<String>),

    #[error("ABI expects the parameter `{0}`, but this was not found")]
    MissingParam(String),
    // #[error(
    //     "Could not read witness value at index {witness_index:?} (required for parameter \"{name}\")"
    // )]
    // MissingParamWitnessValue { name: String, witness_index: Witness },
    // #[error("Attempted to write to witness index {0:?} but it is already initialized to a different value")]
    // InconsistentWitnessAssignment(Witness),

    // #[error("No return value is expected but received {0:?}")]
    // UnexpectedReturnValue(InputValue),
}
