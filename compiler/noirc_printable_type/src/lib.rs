#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]

use std::{collections::BTreeMap, str};

use acvm::{AcirField, acir::brillig::ForeignCallParam};

use iter_extended::vecmap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum PrintableType {
    Field,
    Array {
        length: u32,
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
    Enum {
        name: String,
        variants: Vec<(String, Vec<PrintableType>)>,
    },
    String {
        length: u32,
    },
    FmtString {
        length: u32,
        typ: Box<PrintableType>,
    },
    Function {
        arguments: Vec<PrintableType>,
        return_type: Box<PrintableType>,
        env: Box<PrintableType>,
        unconstrained: bool,
    },
    Reference {
        typ: Box<PrintableType>,
        mutable: bool,
    },
    Unit,
}

/// This is what all formats eventually transform into
/// For example, a toml file will parse into TomlTypes
/// and those TomlTypes will be mapped to Value
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum PrintableValue<F> {
    Field(F),
    String(String),
    FmtString(String, Vec<PrintableValue<F>>),
    Vec { array_elements: Vec<PrintableValue<F>>, is_slice: bool },
    Struct(BTreeMap<String, PrintableValue<F>>),
    Other,
}

/// In order to display a `PrintableValue` we need a `PrintableType` to accurately
/// convert the value into a human-readable format.
pub enum PrintableValueDisplay<F> {
    Plain(PrintableValue<F>, PrintableType),
    FmtString(String, Vec<(PrintableValue<F>, PrintableType)>),
}
impl<F: AcirField> std::fmt::Display for PrintableValueDisplay<F> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Plain(value, typ) => {
                let output_string = to_string(value, typ).ok_or(std::fmt::Error)?;
                write!(fmt, "{output_string}")
            }
            Self::FmtString(template, values) => {
                let mut values_iter = values.iter();
                write_template_replacing_interpolations(template, fmt, || {
                    values_iter.next().and_then(|(value, typ)| to_string(value, typ))
                })
            }
        }
    }
}

fn to_string<F: AcirField>(value: &PrintableValue<F>, typ: &PrintableType) -> Option<String> {
    let mut output = String::new();
    match (value, typ) {
        (PrintableValue::Field(f), PrintableType::Field) => {
            output.push_str(&format_field_string(*f));
        }
        (PrintableValue::Field(f), PrintableType::UnsignedInteger { width }) => {
            // Retain the lower 'width' bits
            debug_assert!(
                *width <= 128,
                "We don't currently support unsigned integers larger than u128"
            );
            let mut uint_cast = f.to_u128();
            if *width != 128 {
                uint_cast &= (1 << width) - 1;
            };

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
            output.push_str(&format!("<<fn({arguments:?}) -> {return_type:?}>>",));
        }
        (_, PrintableType::Reference { mutable: false, .. }) => {
            output.push_str("<<ref>>");
        }
        (_, PrintableType::Reference { mutable: true, .. }) => {
            output.push_str("<<mutable ref>>");
        }
        (PrintableValue::Vec { array_elements, is_slice }, PrintableType::Array { typ, .. })
        | (PrintableValue::Vec { array_elements, is_slice }, PrintableType::Slice { typ }) => {
            if *is_slice {
                output.push('&');
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

        (PrintableValue::FmtString(template, values), PrintableType::FmtString { typ, .. }) => {
            let PrintableType::Tuple { types } = typ.as_ref() else {
                panic!("Expected type to be a Tuple for FmtString");
            };
            let template = template.to_string();
            let args = values.iter().cloned().zip(types.iter().cloned()).collect::<Vec<_>>();
            output.push_str(&PrintableValueDisplay::FmtString(template, args).to_string());
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
            let mut elements = array_elements.iter().zip(types).peekable();
            while let Some((value, typ)) = elements.next() {
                output.push_str(
                    &PrintableValueDisplay::Plain(value.clone(), typ.clone()).to_string(),
                );
                if elements.peek().is_some() {
                    output.push_str(", ");
                }
            }
            if types.len() == 1 {
                output.push(',');
            }
            output.push(')');
        }

        (_, PrintableType::Unit) => output.push_str("()"),

        _ => return None,
    };

    Some(output)
}

fn write_template_replacing_interpolations(
    template: &str,
    fmt: &mut std::fmt::Formatter<'_>,
    mut replacement: impl FnMut() -> Option<String>,
) -> std::fmt::Result {
    let mut last_index = 0; // How far we've written from the template
    let mut char_indices = template.char_indices().peekable();
    while let Some((char_index, char)) = char_indices.next() {
        // If we see a '}' it must be "}}" because the ones for interpolation are handled
        // when we see '{'
        if char == '}' {
            // Write what we've seen so far in the template, including this '}'
            write!(fmt, "{}", &template[last_index..=char_index])?;

            // Skip the second '}'
            let (_, closing_curly) = char_indices.next().unwrap();
            assert_eq!(closing_curly, '}');

            last_index = char_indices.peek().map(|(index, _)| *index).unwrap_or(template.len());
            continue;
        }

        // Keep going forward until we find a '{'
        if char != '{' {
            continue;
        }

        // We'll either have to write an interpolation or '{{' if it's an escape,
        // so let's write what we've seen so far in the template.
        write!(fmt, "{}", &template[last_index..char_index])?;

        // If it's '{{', write '{' and keep going
        if char_indices.peek().map(|(_, char)| char) == Some(&'{') {
            write!(fmt, "{{")?;

            // Skip the second '{'
            char_indices.next().unwrap();

            last_index = char_indices.peek().map(|(index, _)| *index).unwrap_or(template.len());
            continue;
        }

        // Write the interpolation
        if let Some(string) = replacement() {
            write!(fmt, "{string}")?;
        } else {
            return Err(std::fmt::Error);
        }

        // Whatever was inside '{...}' doesn't matter, so skip until we find '}'
        while let Some((_, char)) = char_indices.next() {
            if char == '}' {
                last_index = char_indices.peek().map(|(index, _)| *index).unwrap_or(template.len());
                break;
            }
        }
    }

    write!(fmt, "{}", &template[last_index..])
}

/// This trims any leading zeroes.
/// A singular '0' will be prepended as well if the trimmed string has an odd length.
/// A hex string's length needs to be even to decode into bytes, as two digits correspond to
/// one byte.
pub fn format_field_string<F: AcirField>(field: F) -> String {
    if field.is_zero() {
        return "0x00".to_owned();
    }
    let mut trimmed_field = field.to_hex().trim_start_matches('0').to_owned();
    if trimmed_field.len() % 2 != 0 {
        trimmed_field = "0".to_owned() + &trimmed_field;
    }
    "0x".to_owned() + &trimmed_field
}

/// Assumes that `field_iterator` contains enough field elements in order to decode the [PrintableType]
pub fn decode_printable_value<F: AcirField>(
    field_iterator: &mut impl Iterator<Item = F>,
    typ: &PrintableType,
) -> PrintableValue<F> {
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
                array_elements.push(decode_printable_value(field_iterator, typ));
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
                array_elements.push(decode_printable_value(field_iterator, typ));
            }

            PrintableValue::Vec { array_elements, is_slice: true }
        }
        PrintableType::Tuple { types } => PrintableValue::Vec {
            array_elements: vecmap(types, |typ| decode_printable_value(field_iterator, typ)),
            is_slice: false,
        },
        PrintableType::String { length } => {
            let field_elements: Vec<F> = field_iterator.take(*length as usize).collect();
            let string = decode_string_value(&field_elements);
            PrintableValue::String(string)
        }
        PrintableType::FmtString { length, typ } => {
            // First comes the template string, for which we know its length
            let field_elements: Vec<F> = field_iterator.take(*length as usize).collect();
            let string = decode_string_value(&field_elements);

            // Next comes the number of interpolated values
            let tuple_length =
                field_iterator.next().expect("Expected tuple length").to_u128() as usize;
            let PrintableType::Tuple { types } = typ.as_ref() else {
                panic!("Expected type to be a Tuple for FmtString");
            };
            assert_eq!(tuple_length, types.len());

            // Next come the interpolated values
            let values = types
                .iter()
                .map(|typ| decode_printable_value(field_iterator, typ))
                .collect::<Vec<_>>();

            PrintableValue::FmtString(string, values)
        }
        PrintableType::Struct { fields, .. } => {
            let mut struct_map = BTreeMap::new();

            for (field_key, param_type) in fields {
                let field_value = decode_printable_value(field_iterator, param_type);

                struct_map.insert(field_key.to_owned(), field_value);
            }

            PrintableValue::Struct(struct_map)
        }
        PrintableType::Function { env, .. } => {
            // we want to consume the fields from the environment, but for now they are not actually printed
            let _env = decode_printable_value(field_iterator, env);
            let func_id = field_iterator.next().unwrap();
            PrintableValue::Field(func_id)
        }
        PrintableType::Reference { typ, .. } => {
            // we decode the reference, but it's not really used for printing
            decode_printable_value(field_iterator, typ)
        }
        PrintableType::Unit => PrintableValue::Field(F::zero()),
        PrintableType::Enum { name: _, variants } => {
            let tag = field_iterator.next().unwrap();
            let tag_value = tag.to_u128() as usize;

            let (_name, variant_types) = &variants[tag_value];
            PrintableValue::Vec {
                array_elements: vecmap(variant_types, |typ| {
                    decode_printable_value(field_iterator, typ)
                }),
                is_slice: false,
            }
        }
    }
}

pub fn decode_string_value<F: AcirField>(field_elements: &[F]) -> String {
    let string_as_slice = vecmap(field_elements, |e| {
        let mut field_as_bytes = e.to_be_bytes();
        let char_byte = field_as_bytes.pop().unwrap(); // A character in a string is represented by a u8, thus we just want the last byte of the element
        assert!(field_as_bytes.into_iter().all(|b| b == 0)); // Assert that the rest of the field element's bytes are empty
        char_byte
    });

    let final_string = str::from_utf8(&string_as_slice).unwrap();
    final_string.to_owned()
}

pub enum TryFromParamsError {
    MissingForeignCallInputs,
    ParsingError(serde_json::Error),
}

impl<F: AcirField> PrintableValueDisplay<F> {
    pub fn try_from_params(
        foreign_call_inputs: &[ForeignCallParam<F>],
    ) -> Result<PrintableValueDisplay<F>, TryFromParamsError> {
        let (is_fmt_str, foreign_call_inputs) =
            foreign_call_inputs.split_last().ok_or(TryFromParamsError::MissingForeignCallInputs)?;

        if is_fmt_str.unwrap_field().is_one() {
            convert_fmt_string_inputs(foreign_call_inputs)
        } else {
            convert_string_inputs(foreign_call_inputs)
        }
    }
}

fn convert_string_inputs<F: AcirField>(
    foreign_call_inputs: &[ForeignCallParam<F>],
) -> Result<PrintableValueDisplay<F>, TryFromParamsError> {
    // Fetch the PrintableType from the foreign call input
    // The remaining input values should hold what is to be printed
    let (printable_type_as_values, input_values) =
        foreign_call_inputs.split_last().ok_or(TryFromParamsError::MissingForeignCallInputs)?;
    let printable_type = fetch_printable_type(printable_type_as_values)?;

    // We must use a flat map here as each value in a struct will be in a separate input value
    let mut input_values_as_fields = input_values.iter().flat_map(|param| param.fields());

    let value = decode_printable_value(&mut input_values_as_fields, &printable_type);

    Ok(PrintableValueDisplay::Plain(value, printable_type))
}

fn convert_fmt_string_inputs<F: AcirField>(
    foreign_call_inputs: &[ForeignCallParam<F>],
) -> Result<PrintableValueDisplay<F>, TryFromParamsError> {
    let (message, input_and_printable_types) =
        foreign_call_inputs.split_first().ok_or(TryFromParamsError::MissingForeignCallInputs)?;

    let message_as_fields = message.fields();
    let message_as_string = decode_string_value(&message_as_fields);

    let (num_values, input_and_printable_types) = input_and_printable_types
        .split_first()
        .ok_or(TryFromParamsError::MissingForeignCallInputs)?;

    let mut output = Vec::new();
    let num_values = num_values.unwrap_field().to_u128() as usize;

    let types_start_at = input_and_printable_types.len() - num_values;

    let mut input_iter =
        input_and_printable_types[0..types_start_at].iter().flat_map(|param| param.fields());
    for printable_type in input_and_printable_types.iter().skip(types_start_at) {
        let printable_type = fetch_printable_type(printable_type)?;
        let value = decode_printable_value(&mut input_iter, &printable_type);

        output.push((value, printable_type));
    }

    Ok(PrintableValueDisplay::FmtString(message_as_string, output))
}

fn fetch_printable_type<F: AcirField>(
    printable_type: &ForeignCallParam<F>,
) -> Result<PrintableType, TryFromParamsError> {
    let printable_type_as_fields = printable_type.fields();
    let printable_type_as_string = decode_string_value(&printable_type_as_fields);
    let printable_type: Result<PrintableType, serde_json::Error> =
        serde_json::from_str(&printable_type_as_string);
    match printable_type {
        Ok(printable_type) => Ok(printable_type),
        Err(err) => Err(TryFromParamsError::ParsingError(err)),
    }
}

#[cfg(test)]
mod tests {
    use acvm::FieldElement;

    use proptest::prelude::*;

    use crate::to_string;

    use super::{PrintableType, PrintableValue, PrintableValueDisplay};

    #[test]
    fn printable_value_display_to_string_without_interpolations() {
        let template = "hello";
        let display =
            PrintableValueDisplay::<FieldElement>::FmtString(template.to_string(), vec![]);
        assert_eq!(display.to_string(), template);
    }

    #[test]
    fn printable_value_display_to_string_with_curly_escapes() {
        let template = "hello {{world}} {{{{double_escape}}}}";
        let expected = "hello {world} {{double_escape}}";
        let display =
            PrintableValueDisplay::<FieldElement>::FmtString(template.to_string(), vec![]);
        assert_eq!(display.to_string(), expected);
    }

    #[test]
    fn printable_value_display_to_string_with_interpolations() {
        let template = "hello {one} {{no}} {two} {{not_again}} {three} world";
        let values = vec![
            (PrintableValue::String("ONE".to_string()), PrintableType::String { length: 3 }),
            (PrintableValue::String("TWO".to_string()), PrintableType::String { length: 3 }),
            (PrintableValue::String("THREE".to_string()), PrintableType::String { length: 5 }),
        ];
        let expected = "hello ONE {no} TWO {not_again} THREE world";
        let display =
            PrintableValueDisplay::<FieldElement>::FmtString(template.to_string(), values);
        assert_eq!(display.to_string(), expected);
    }

    #[test]
    fn one_element_tuple_to_string() {
        let value = PrintableValue::<FieldElement>::Vec {
            array_elements: vec![PrintableValue::Field(1_u128.into())],
            is_slice: false,
        };
        let typ = PrintableType::Tuple { types: vec![PrintableType::Field] };
        let string = to_string(&value, &typ);
        assert_eq!(string.unwrap(), "(0x01,)");
    }

    #[test]
    fn two_elements_tuple_to_string() {
        let value = PrintableValue::<FieldElement>::Vec {
            array_elements: vec![
                PrintableValue::Field(1_u128.into()),
                PrintableValue::Field(2_u128.into()),
            ],
            is_slice: false,
        };
        let typ = PrintableType::Tuple { types: vec![PrintableType::Field, PrintableType::Field] };
        let string = to_string(&value, &typ);
        assert_eq!(string.unwrap(), "(0x01, 0x02)");
    }

    proptest! {
        #[test]
        fn handles_decoding_u128_values(uint_value: u128) {
            let value = PrintableValue::Field(FieldElement::from(uint_value));
            let typ = PrintableType::UnsignedInteger { width: 128 };

            let value_as_string = to_string(&value, &typ).unwrap();
            // We want to match rust's stringification.
            prop_assert_eq!(value_as_string, uint_value.to_string());
        }
    }
}
