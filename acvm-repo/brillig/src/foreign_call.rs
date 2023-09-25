use crate::value::Value;
use serde::{Deserialize, Serialize};

/// Single output of a [foreign call][crate::Opcode::ForeignCall].
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub enum ForeignCallOutput {
    Single(Value),
    Array(Vec<Value>),
}

/// Represents the full output of a [foreign call][crate::Opcode::ForeignCall].
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct ForeignCallResult {
    /// Resolved output values of the foreign call.
    pub values: Vec<ForeignCallOutput>,
}

impl From<Value> for ForeignCallResult {
    fn from(value: Value) -> Self {
        ForeignCallResult { values: vec![ForeignCallOutput::Single(value)] }
    }
}

impl From<Vec<Value>> for ForeignCallResult {
    fn from(values: Vec<Value>) -> Self {
        ForeignCallResult { values: vec![ForeignCallOutput::Array(values)] }
    }
}

impl From<Vec<ForeignCallOutput>> for ForeignCallResult {
    fn from(values: Vec<ForeignCallOutput>) -> Self {
        ForeignCallResult { values }
    }
}
