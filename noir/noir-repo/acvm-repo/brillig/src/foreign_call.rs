use crate::value::Value;
use serde::{Deserialize, Serialize};

/// Single output of a [foreign call][crate::Opcode::ForeignCall].
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub enum ForeignCallParam {
    Single(Value),
    Array(Vec<Value>),
}

impl From<Value> for ForeignCallParam {
    fn from(value: Value) -> Self {
        ForeignCallParam::Single(value)
    }
}

impl From<Vec<Value>> for ForeignCallParam {
    fn from(values: Vec<Value>) -> Self {
        ForeignCallParam::Array(values)
    }
}

impl ForeignCallParam {
    pub fn values(&self) -> Vec<Value> {
        match self {
            ForeignCallParam::Single(value) => vec![*value],
            ForeignCallParam::Array(values) => values.clone(),
        }
    }

    pub fn unwrap_value(&self) -> Value {
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

impl From<Value> for ForeignCallResult {
    fn from(value: Value) -> Self {
        ForeignCallResult { values: vec![value.into()] }
    }
}

impl From<Vec<Value>> for ForeignCallResult {
    fn from(values: Vec<Value>) -> Self {
        ForeignCallResult { values: vec![values.into()] }
    }
}

impl From<Vec<ForeignCallParam>> for ForeignCallResult {
    fn from(values: Vec<ForeignCallParam>) -> Self {
        ForeignCallResult { values }
    }
}
