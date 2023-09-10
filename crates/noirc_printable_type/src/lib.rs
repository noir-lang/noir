use std::{collections::BTreeMap, str};

use acvm::{brillig_vm::brillig::Value, FieldElement};
use iter_extended::vecmap;
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use thiserror::Error;

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
    fn field_count(&self) -> u32 {
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
pub enum PrintableValueDisplay {
    Plain(PrintableValue, PrintableType),
    FmtString(String, Vec<(PrintableValue, PrintableType)>),
}

#[derive(Debug, Error)]
pub enum ForeignCallError {
    #[error("Foreign call inputs needed for execution are missing")]
    MissingForeignCallInputs,

    #[error("Could not parse PrintableType argument. {0}")]
    ParsingError(#[from] serde_json::Error),
}

impl TryFrom<&[Vec<Value>]> for PrintableValueDisplay {
    type Error = ForeignCallError;

    fn try_from(foreign_call_inputs: &[Vec<Value>]) -> Result<Self, Self::Error> {
        let (is_fmt_str, foreign_call_inputs) =
            foreign_call_inputs.split_last().ok_or(ForeignCallError::MissingForeignCallInputs)?;

        if is_fmt_str[0].to_field().is_one() {
            convert_fmt_string_inputs(foreign_call_inputs)
        } else {
            convert_string_inputs(foreign_call_inputs)
        }
    }
}

fn convert_string_inputs(
    foreign_call_inputs: &[Vec<Value>],
) -> Result<PrintableValueDisplay, ForeignCallError> {
    // Fetch the PrintableType from the foreign call input
    // The remaining input values should hold what is to be printed
    let (printable_type_as_values, input_values) =
        foreign_call_inputs.split_last().ok_or(ForeignCallError::MissingForeignCallInputs)?;
    let printable_type = fetch_printable_type(printable_type_as_values)?;

    // We must use a flat map here as each value in a struct will be in a separate input value
    let mut input_values_as_fields =
        input_values.iter().flat_map(|values| vecmap(values, |value| value.to_field()));

    let value = decode_value(&mut input_values_as_fields, &printable_type);

    Ok(PrintableValueDisplay::Plain(value, printable_type))
}

fn convert_fmt_string_inputs(
    foreign_call_inputs: &[Vec<Value>],
) -> Result<PrintableValueDisplay, ForeignCallError> {
    let (message_as_values, input_and_printable_values) =
        foreign_call_inputs.split_first().ok_or(ForeignCallError::MissingForeignCallInputs)?;

    let message_as_fields = vecmap(message_as_values, |value| value.to_field());
    let message_as_string = decode_string_value(&message_as_fields);

    let (num_values, input_and_printable_values) = input_and_printable_values
        .split_first()
        .ok_or(ForeignCallError::MissingForeignCallInputs)?;

    let mut output = Vec::new();
    let num_values = num_values[0].to_field().to_u128() as usize;

    for (i, printable_value) in input_and_printable_values
        .iter()
        .skip(input_and_printable_values.len() - num_values)
        .enumerate()
    {
        let printable_type = fetch_printable_type(printable_value)?;
        let type_size = printable_type.field_count() as usize;

        let mut input_values_as_fields = input_and_printable_values[i..(i + type_size)]
            .iter()
            .flat_map(|values| vecmap(values, |value| value.to_field()));

        let value = decode_value(&mut input_values_as_fields, &printable_type);

        output.push((value, printable_type));
    }

    Ok(PrintableValueDisplay::FmtString(message_as_string, output))
}

fn fetch_printable_type(
    printable_type_as_values: &[Value],
) -> Result<PrintableType, ForeignCallError> {
    let printable_type_as_fields = vecmap(printable_type_as_values, |value| value.to_field());
    let printable_type_as_string = decode_string_value(&printable_type_as_fields);
    let printable_type: PrintableType = serde_json::from_str(&printable_type_as_string)?;

    Ok(printable_type)
}

fn to_string(value: &PrintableValue, typ: &PrintableType) -> Option<String> {
    let mut output = String::new();
    match (value, typ) {
        (PrintableValue::Field(f), PrintableType::Field) => {
            output.push_str(&format_field_string(*f));
        }
        (
            PrintableValue::Field(f),
            PrintableType::UnsignedInteger { .. },
        ) => {
            output.push_str(&format!("{}", f.to_u128()));
        }
        (PrintableValue::Field(_f), PrintableType::SignedInteger { .. }) => {
            // TODO: I am not sure how to convert an element to a signed integer in this case 
        }
        (PrintableValue::Field(f), PrintableType::Boolean) => {
            if f.is_one() {
                output.push_str("true");
            } else {
                output.push_str("false");
            }
        }
        (PrintableValue::Vec(vector), PrintableType::Array { typ, .. }) => {
            output.push('[');
            let mut values = vector.iter().peekable();
            while let Some(value) = values.next() {
                output.push_str(&format!(
                    "{}",
                    PrintableValueDisplay::Plain(value.clone(), *typ.clone())
                ));
                if values.peek().is_some() {
                    output.push_str(", ");
                }
            }
            output.push(']');
        }

        (PrintableValue::String(s), PrintableType::String { .. }) => {
            output.push_str(&format!(r#""{s}""#));
        }

        (PrintableValue::Struct(map), PrintableType::Struct { name, fields, .. }) => {
            output.push_str(&format!("{name} {{ "));

            let mut fields = fields.iter().peekable();
            while let Some((key, field_type)) = fields.next() {
                let value = &map[key];
                output.push_str(&format!(
                    "{key}: {}",
                    PrintableValueDisplay::Plain(value.clone(), field_type.clone())
                ));
                if fields.peek().is_some() {
                    output.push_str(", ");
                }
            }

            output.push_str(" }");
        }

        _ => return None,
    };

    Some(output)
}

// Taken from Regex docs directly
fn replace_all<E>(
    re: &Regex,
    haystack: &str,
    mut replacement: impl FnMut(&Captures) -> Result<String, E>,
) -> Result<String, E> {
    let mut new = String::with_capacity(haystack.len());
    let mut last_match = 0;
    for caps in re.captures_iter(haystack) {
        let m = caps.get(0).unwrap();
        new.push_str(&haystack[last_match..m.start()]);
        new.push_str(&replacement(&caps)?);
        last_match = m.end();
    }
    new.push_str(&haystack[last_match..]);
    Ok(new)
}

impl std::fmt::Display for PrintableValueDisplay {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Plain(value, typ) => {
                let output_string = to_string(value, typ).ok_or(std::fmt::Error)?;
                write!(fmt, "{output_string}")
            }
            Self::FmtString(template, values) => {
                let mut display_iter = values.iter();
                let re = Regex::new(r"\{([a-zA-Z0-9_]+)\}").map_err(|_| std::fmt::Error)?;

                let formatted_str = replace_all(&re, template, |_: &Captures| {
                    let (value, typ) = display_iter.next().ok_or(std::fmt::Error)?;
                    to_string(value, typ).ok_or(std::fmt::Error)
                })?;

                write!(fmt, "{formatted_str}")
            }
        }
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

/// Assumes that `field_iterator` contains enough [FieldElement] in order to decode the [PrintableType]
fn decode_value(
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

fn decode_string_value(field_elements: &[FieldElement]) -> String {
    // TODO: Replace with `into` when Char is supported
    let string_as_slice = vecmap(field_elements, |e| {
        let mut field_as_bytes = e.to_be_bytes();
        let char_byte = field_as_bytes.pop().unwrap(); // A character in a string is represented by a u8, thus we just want the last byte of the element
        assert!(field_as_bytes.into_iter().all(|b| b == 0)); // Assert that the rest of the field element's bytes are empty
        char_byte
    });

    let final_string = str::from_utf8(&string_as_slice).unwrap();
    final_string.to_owned()
}
