use acir_field::FieldElement;
use serde::{Deserialize, Serialize};

/// Single output of a [foreign call][crate::Opcode::ForeignCall].
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub enum ForeignCallParam {
    Single(FieldElement),
    Array(Vec<FieldElement>),
}

impl From<FieldElement> for ForeignCallParam {
    fn from(value: FieldElement) -> Self {
        ForeignCallParam::Single(value)
    }
}

impl From<Vec<FieldElement>> for ForeignCallParam {
    fn from(values: Vec<FieldElement>) -> Self {
        ForeignCallParam::Array(values)
    }
}

impl ForeignCallParam {
    pub fn fields(&self) -> Vec<FieldElement> {
        match self {
            ForeignCallParam::Single(value) => vec![*value],
            ForeignCallParam::Array(values) => values.clone(),
        }
    }

    pub fn unwrap_field(&self) -> FieldElement {
        match self {
            ForeignCallParam::Single(value) => *value,
            ForeignCallParam::Array(_) => panic!("Expected single value, found array"),
        }
    }
}

/// Represents the full output of a [foreign call][crate::Opcode::ForeignCall].
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Default)]
pub struct ForeignCallResult {
    /// Resolved output values of the foreign call.
    pub values: Vec<ForeignCallParam>,
}

impl From<FieldElement> for ForeignCallResult {
    fn from(value: FieldElement) -> Self {
        ForeignCallResult { values: vec![value.into()] }
    }
}

impl From<Vec<FieldElement>> for ForeignCallResult {
    fn from(values: Vec<FieldElement>) -> Self {
        ForeignCallResult { values: vec![values.into()] }
    }
}

impl From<Vec<ForeignCallParam>> for ForeignCallResult {
    fn from(values: Vec<ForeignCallParam>) -> Self {
        ForeignCallResult { values }
    }
}
