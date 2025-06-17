#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]

use std::{collections::BTreeMap, str};
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
    Enum {
        name: String,
        variants: Vec<(String, Option<PrintableType>)>,
    },
    Reference {
        typ: Box<PrintableType>,
        mutable: bool,
    },
    Unit,
}

/// PrintableValue represents runtime values that can be printed
/// For now this is a placeholder - will be defined based on Sensei's needs
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrintableValue {
    Field(String),
    String(String),
    Boolean(bool),
    Array(Vec<PrintableValue>),
    Tuple(Vec<PrintableValue>),
    Struct(BTreeMap<String, PrintableValue>),
    Unit,
}

impl PrintableType {
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            PrintableType::Field
                | PrintableType::SignedInteger { .. }
                | PrintableType::UnsignedInteger { .. }
        )
    }
}

/// Placeholder display implementation
pub struct PrintableValueDisplay {
    value: PrintableValue,
}

impl std::fmt::Display for PrintableValueDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.value {
            PrintableValue::Field(s) => write!(f, "{}", s),
            PrintableValue::String(s) => write!(f, "{}", s),
            PrintableValue::Boolean(b) => write!(f, "{}", b),
            PrintableValue::Unit => write!(f, "()"),
            PrintableValue::Array(values) => {
                write!(f, "[")?;
                for (i, val) in values.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", PrintableValueDisplay { value: val.clone() })?;
                }
                write!(f, "]")
            }
            PrintableValue::Tuple(values) => {
                write!(f, "(")?;
                for (i, val) in values.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", PrintableValueDisplay { value: val.clone() })?;
                }
                write!(f, ")")
            }
            PrintableValue::Struct(fields) => {
                write!(f, "{{")?;
                for (i, (name, val)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", name, PrintableValueDisplay { value: val.clone() })?;
                }
                write!(f, "}}")
            }
        }
    }
}

/// Utility function to format a field value as a string
pub fn format_field_string(field: &str) -> String {
    field.to_string()
}

/// Decode a string value from field elements
pub fn decode_string_value(values: &[String]) -> String {
    values.join("")
}