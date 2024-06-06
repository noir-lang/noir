use acir_field::AcirField;
use serde::{Deserialize, Serialize};

/// Single output of a [foreign call][crate::Opcode::ForeignCall].
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
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
    pub fn fields(&self) -> Vec<F> {
        match self {
            ForeignCallParam::Single(value) => vec![*value],
            ForeignCallParam::Array(values) => values.to_vec(),
        }
    }

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

impl<F> From<F> for ForeignCallResult<F> {
    fn from(value: F) -> Self {
        ForeignCallResult { values: vec![value.into()] }
    }
}

impl<F> From<Vec<F>> for ForeignCallResult<F> {
    fn from(values: Vec<F>) -> Self {
        ForeignCallResult { values: vec![values.into()] }
    }
}

impl<F> From<Vec<ForeignCallParam<F>>> for ForeignCallResult<F> {
    fn from(values: Vec<ForeignCallParam<F>>) -> Self {
        ForeignCallResult { values }
    }
}
