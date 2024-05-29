use std::{collections::BTreeMap, str};

use acvm::{acir::AcirField, brillig_vm::brillig::ForeignCallParam, FieldElement};
use iter_extended::vecmap;
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum PrintableType {
    Field,
    Array {
        length: u64,
        #[serde(rename = "type")]
        typ: Box<PrintableType>,
    },
    Slice {
        #[serde(rename = "type")]
        typ: Box<PrintableType>,
    },
    Tuple {
        types: Vec<PrintableType>,
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
    Function {
        arguments: Vec<PrintableType>,
        return_type: Box<PrintableType>,
        env: Box<PrintableType>,
    },
    MutableReference {
        typ: Box<PrintableType>,
    },
    Unit,
}

/// This is what all formats eventually transform into
/// For example, a toml file will parse into TomlTypes
/// and those TomlTypes will be mapped to Value
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum PrintableValue {
    Field(FieldElement),
    String(String),
    Vec { array_elements: Vec<PrintableValue>, is_slice: bool },
    Struct(BTreeMap<String, PrintableValue>),
    Other,
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

    #[error("Failed calling external resolver. {0}")]
    ExternalResolverError(#[from] jsonrpc::Error),

    #[error("Assert message resolved after an unsatisified constrain. {0}")]
    ResolvedAssertMessage(String),
}

impl TryFrom<&[ForeignCallParam<FieldElement>]> for PrintableValueDisplay {
    type Error = ForeignCallError;

    fn try_from(
        foreign_call_inputs: &[ForeignCallParam<FieldElement>],
    ) -> Result<Self, Self::Error> {
        let (is_fmt_str, foreign_call_inputs) =
            foreign_call_inputs.split_last().ok_or(ForeignCallError::MissingForeignCallInputs)?;

        if is_fmt_str.unwrap_field().is_one() {
            convert_fmt_string_inputs(foreign_call_inputs)
        } else {
            convert_string_inputs(foreign_call_inputs)
        }
    }
}

fn convert_string_inputs(
    foreign_call_inputs: &[ForeignCallParam<FieldElement>],
) -> Result<PrintableValueDisplay, ForeignCallError> {
    // Fetch the PrintableType from the foreign call input
    // The remaining input values should hold what is to be printed
    let (printable_type_as_values, input_values) =
        foreign_call_inputs.split_last().ok_or(ForeignCallError::MissingForeignCallInputs)?;
    let printable_type = fetch_printable_type(printable_type_as_values)?;

    // We must use a flat map here as each value in a struct will be in a separate input value
    let mut input_values_as_fields = input_values.iter().flat_map(|param| param.fields());

    let value = decode_value(&mut input_values_as_fields, &printable_type);

    Ok(PrintableValueDisplay::Plain(value, printable_type))
}

fn convert_fmt_string_inputs(
    foreign_call_inputs: &[ForeignCallParam<FieldElement>],
) -> Result<PrintableValueDisplay, ForeignCallError> {
    let (message, input_and_printable_types) =
        foreign_call_inputs.split_first().ok_or(ForeignCallError::MissingForeignCallInputs)?;

    let message_as_fields = message.fields();
    let message_as_string = decode_string_value(&message_as_fields);

    let (num_values, input_and_printable_types) = input_and_printable_types
        .split_first()
        .ok_or(ForeignCallError::MissingForeignCallInputs)?;

    let mut output = Vec::new();
    let num_values = num_values.unwrap_field().to_u128() as usize;

    let types_start_at = input_and_printable_types.len() - num_values;
    let mut input_iter =
        input_and_printable_types[0..types_start_at].iter().flat_map(|param| param.fields());
    for printable_type in input_and_printable_types.iter().skip(types_start_at) {
        let printable_type = fetch_printable_type(printable_type)?;
        let value = decode_value(&mut input_iter, &printable_type);

        output.push((value, printable_type));
    }

    Ok(PrintableValueDisplay::FmtString(message_as_string, output))
}

fn fetch_printable_type(
    printable_type: &ForeignCallParam<FieldElement>,
) -> Result<PrintableType, ForeignCallError> {
    let printable_type_as_fields = printable_type.fields();
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
        (PrintableValue::Field(f), PrintableType::UnsignedInteger { width }) => {
            let uint_cast = f.to_u128() & ((1 << width) - 1); // Retain the lower 'width' bits
            output.push_str(&uint_cast.to_string());
        }
        (PrintableValue::Field(f), PrintableType::SignedInteger { width }) => {
            let mut uint = f.to_u128(); // Interpret as uint

            // Extract sign relative to width of input
            if (uint >> (width - 1)) == 1 {
                output.push('-');
                uint = (uint ^ ((1 << width) - 1)) + 1; // Two's complement relative to width of input
            }

            output.push_str(&uint.to_string());
        }
        (PrintableValue::Field(f), PrintableType::Boolean) => {
            if f.is_one() {
                output.push_str("true");
            } else {
                output.push_str("false");
            }
        }
        (PrintableValue::Field(_), PrintableType::Function { arguments, return_type, .. }) => {
            output.push_str(&format!("<<fn({:?}) -> {:?}>>", arguments, return_type,));
        }
        (_, PrintableType::MutableReference { .. }) => {
            output.push_str("<<mutable ref>>");
        }
        (PrintableValue::Vec { array_elements, is_slice }, PrintableType::Array { typ, .. })
        | (PrintableValue::Vec { array_elements, is_slice }, PrintableType::Slice { typ }) => {
            if *is_slice {
                output.push('&')
            }
            output.push('[');
            let mut values = array_elements.iter().peekable();
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
            output.push_str(s);
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

        (PrintableValue::Vec { array_elements, .. }, PrintableType::Tuple { types }) => {
            output.push('(');
            let mut elems = array_elements.iter().zip(types).peekable();
            while let Some((value, typ)) = elems.next() {
                output.push_str(
                    &PrintableValueDisplay::Plain(value.clone(), typ.clone()).to_string(),
                );
                if elems.peek().is_some() {
                    output.push_str(", ");
                }
            }
            output.push(')');
        }

        (_, PrintableType::Unit) => output.push_str("()"),

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

            PrintableValue::Vec { array_elements, is_slice: false }
        }
        PrintableType::Slice { typ } => {
            let length = field_iterator
                .next()
                .expect("not enough data to decode variable array length")
                .to_u128() as usize;
            let mut array_elements = Vec::with_capacity(length);
            for _ in 0..length {
                array_elements.push(decode_value(field_iterator, typ));
            }

            PrintableValue::Vec { array_elements, is_slice: true }
        }
        PrintableType::Tuple { types } => PrintableValue::Vec {
            array_elements: vecmap(types, |typ| decode_value(field_iterator, typ)),
            is_slice: false,
        },
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
        PrintableType::Function { env, .. } => {
            let field_element = field_iterator.next().unwrap();
            let func_ref = PrintableValue::Field(field_element);
            // we want to consume the fields from the environment, but for now they are not actually printed
            decode_value(field_iterator, env);
            func_ref
        }
        PrintableType::MutableReference { typ } => {
            // we decode the reference, but it's not really used for printing
            decode_value(field_iterator, typ)
        }
        PrintableType::Unit => PrintableValue::Field(FieldElement::zero()),
    }
}

pub fn decode_string_value(field_elements: &[FieldElement]) -> String {
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
