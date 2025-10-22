use acir_field::AcirField;
use serde::{Deserialize, Serialize};

/// Single input or output of a [foreign call][crate::Opcode::ForeignCall].
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ForeignCallParam<F> {
    Single(F),
    Array(Vec<F>),
}

impl<F> From<F> for ForeignCallParam<F> {
    fn from(value: F) -> Self {
        ForeignCallParam::Single(value)
    }
}

impl<F> From<Vec<F>> for ForeignCallParam<F> {
    fn from(values: Vec<F>) -> Self {
        ForeignCallParam::Array(values)
    }
}

impl<F: AcirField> ForeignCallParam<F> {
    /// Convert the fields in the parameter into a vector, used to flatten data.
    pub fn fields(&self) -> Vec<F> {
        match self {
            ForeignCallParam::Single(value) => vec![*value],
            ForeignCallParam::Array(values) => values.to_vec(),
        }
    }

    /// Unwrap the field in a `Single` input. Panics if it's an `Array`.
    pub fn unwrap_field(&self) -> F {
        match self {
            ForeignCallParam::Single(value) => *value,
            ForeignCallParam::Array(_) => panic!("Expected single value, found array"),
        }
    }
}

/// Represents the full output of a [foreign call][crate::Opcode::ForeignCall].
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Default)]
pub struct ForeignCallResult<F> {
    /// Resolved output values of the foreign call.
    pub values: Vec<ForeignCallParam<F>>,
}

/// Result of a call returning a one output value.
impl<F> From<F> for ForeignCallResult<F> {
    fn from(value: F) -> Self {
        ForeignCallResult { values: vec![value.into()] }
    }
}

/// Result of a call returning a one output array.
impl<F> From<Vec<F>> for ForeignCallResult<F> {
    fn from(values: Vec<F>) -> Self {
        ForeignCallResult { values: vec![values.into()] }
    }
}

/// Result of a call returning multiple outputs.
impl<F> From<Vec<ForeignCallParam<F>>> for ForeignCallResult<F> {
    fn from(values: Vec<ForeignCallParam<F>>) -> Self {
        ForeignCallResult { values }
    }
}
