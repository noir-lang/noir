#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]

use std::{collections::BTreeMap, str};

use acvm::AcirField;

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
    String {
        length: u32,
    },
    Function {
        arguments: Vec<PrintableType>,
        return_type: Box<PrintableType>,
        env: Box<PrintableType>,
        unconstrained: bool,
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
pub enum PrintableValue<F> {
    Field(F),
    String(String),
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
            write!(fmt, "{}", string)?;
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
fn format_field_string<F: AcirField>(field: F) -> String {
    if field.is_zero() {
        return "0x00".to_owned();
    }
    let mut trimmed_field = field.to_hex().trim_start_matches('0').to_owned();
    if trimmed_field.len() % 2 != 0 {
        trimmed_field = "0".to_owned() + &trimmed_field;
    }
    "0x".to_owned() + &trimmed_field
}

#[cfg(test)]
mod tests {
    use acvm::FieldElement;

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
}
