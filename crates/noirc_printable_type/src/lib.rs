use std::{collections::BTreeMap, str};

use acvm::FieldElement;
use iter_extended::vecmap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum PrintableType {
    Field,
    Array {
        length: u64,
        #[serde(rename = "type")]
        typ: Box<PrintableType>,
    },
    SignedInteger {
        width: u32,
    },
    UnsignedInteger {
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
            Self::Field
            | Self::SignedInteger { .. }
            | Self::UnsignedInteger { .. }
            | Self::Boolean => 1,
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

/// In order to display a `PrintableValue` we need a `PrintableType` to accurately
/// convert the value into a human-readable format.
pub struct PrintableValueDisplay<'a> {
    value: &'a PrintableValue,
    typ: &'a PrintableType,
}

impl<'a> PrintableValueDisplay<'a> {
    #[must_use]
    pub fn new(value: &'a PrintableValue, typ: &'a PrintableType) -> Self {
        Self { value, typ }
    }
}

impl<'a> std::fmt::Display for PrintableValueDisplay<'a> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.value, &self.typ) {
            (
                PrintableValue::Field(f),
                PrintableType::Field
                // TODO: We should print the sign for these and probably print normal integers instead of field strings
                | PrintableType::SignedInteger { .. }
                | PrintableType::UnsignedInteger { .. },
            ) => {
                write!(fmt, "{}", format_field_string(*f))?;
            }
            (PrintableValue::Field(f), PrintableType::Boolean) => {
                if f.is_one() {
                    write!(fmt, "true")?;
                } else {
                    write!(fmt, "false")?;
                }
            }
            (PrintableValue::Vec(vector), PrintableType::Array { typ, .. }) => {
                write!(fmt, "[")?;
                let mut values = vector.iter().peekable();
                while let Some(value) = values.next()  {
                    write!(fmt, "{}", PrintableValueDisplay::new(value, typ))?;
                    if values.peek().is_some() {
                        write!(fmt, ", ")?;
                    }
                }
                write!(fmt, "]")?;
            }

            (PrintableValue::String(s), PrintableType::String { .. }) => {
                write!(fmt, r#""{s}""#)?;
            }

            (PrintableValue::Struct(map), PrintableType::Struct { name, fields, .. }) => {
                write!(fmt, "{name} {{ ")?;

                let mut fields = fields.iter().peekable();
                while let Some((key, field_type)) = fields.next()  {
                    let value = &map[key];
                    write!(fmt, "{key}: {}", PrintableValueDisplay::new(value, field_type))?;
                    if fields.peek().is_some() {
                        write!(fmt, ", ")?;
                    }
                }

                write!(fmt, " }}")?;
            }

            _ => return Err(std::fmt::Error),
        };
        Ok(())
    }
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

// TODO: Figure out a better API for this to avoid exporting the function
/// Assumes that `field_iterator` contains enough [FieldElement] in order to decode the [PrintableType]
pub fn decode_value(
    field_iterator: &mut impl Iterator<Item = FieldElement>,
    typ: &PrintableType,
) -> PrintableValue {
    match typ {
        PrintableType::Field
        | PrintableType::SignedInteger { .. }
        | PrintableType::UnsignedInteger { .. }
        | PrintableType::Boolean => {
            let field_element = field_iterator.next().unwrap();

            PrintableValue::Field(field_element)
        }
        PrintableType::Array { length, typ } => {
            let length = *length as usize;
            let mut array_elements = Vec::with_capacity(length);
            for _ in 0..length {
                array_elements.push(decode_value(field_iterator, typ));
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
                let field_value = decode_value(field_iterator, param_type);

                struct_map.insert(field_key.to_owned(), field_value);
            }

            PrintableValue::Struct(struct_map)
        }
    }
}

// TODO: Figure out a better API for this to avoid exporting the function
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
