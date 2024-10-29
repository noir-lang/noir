use std::str::FromStr;

use super::opcodes::BlockId;
use crate::native_types::{Expression, Witness};
use brillig::Opcode as BrilligOpcode;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Inputs for the Brillig VM. These are the initial inputs
/// that the Brillig VM will use to start.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum BrilligInputs<F> {
    Single(Expression<F>),
    Array(Vec<Expression<F>>),
    MemoryArray(BlockId),
}

/// Outputs for the Brillig VM. Once the VM has completed
/// execution, this will be the object that is returned.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum BrilligOutputs {
    Simple(Witness),
    Array(Vec<Witness>),
}

/// This is purely a wrapper struct around a list of Brillig opcode's which represents
/// a full Brillig function to be executed by the Brillig VM.
/// This is stored separately on a program and accessed through a [BrilligPointer].
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Default, Debug)]
pub struct BrilligBytecode<F> {
    pub bytecode: Vec<BrilligOpcode<F>>,
}

/// Id for the function being called.
#[derive(
    Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash, Copy, Default, PartialOrd, Ord,
)]
#[serde(transparent)]
pub struct BrilligFunctionId(pub u32);

impl BrilligFunctionId {
    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

impl std::fmt::Display for BrilligFunctionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Procedures are a set of complex operations that are common in the noir language.
/// Extracting them to reusable procedures allows us to reduce the size of the generated Brillig.
/// Procedures receive their arguments on scratch space to avoid stack dumping&restoring.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, Deserialize, Serialize)]
pub enum ProcedureId {
    ArrayCopy,
    ArrayReverse,
    VectorCopy,
    MemCopy,
    PrepareVectorPush(bool),
    VectorPop(bool),
    PrepareVectorInsert,
    VectorRemove,
    CheckMaxStackDepth,
}

impl std::fmt::Display for ProcedureId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcedureId::ArrayCopy => write!(f, "ArrayCopy"),
            ProcedureId::ArrayReverse => write!(f, "ArrayReverse"),
            ProcedureId::VectorCopy => write!(f, "VectorCopy"),
            ProcedureId::MemCopy => write!(f, "MemCopy"),
            ProcedureId::PrepareVectorPush(flag) => write!(f, "PrepareVectorPush({flag})"),
            ProcedureId::VectorPop(flag) => write!(f, "VectorPop({flag})"),
            ProcedureId::PrepareVectorInsert => write!(f, "PrepareVectorInsert"),
            ProcedureId::VectorRemove => write!(f, "VectorRemove"),
            ProcedureId::CheckMaxStackDepth => write!(f, "CheckMaxStackDepth"),
        }
    }
}
#[derive(Error, Debug)]
pub enum ProcedureIdFromStrError {
    #[error("Invalid procedure id string: {0}")]
    InvalidProcedureIdString(String),
}

/// The implementation of display and FromStr allows serializing and deserializing a ProcedureId to a string.
/// This is useful when used as key in a map that has to be serialized to JSON/TOML, for example when mapping an opcode to its metadata.
impl FromStr for ProcedureId {
    type Err = ProcedureIdFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let res = match s {
            "ArrayCopy" => ProcedureId::ArrayCopy,
            "ArrayReverse" => ProcedureId::ArrayReverse,
            "VectorCopy" => ProcedureId::VectorCopy,
            "MemCopy" => ProcedureId::MemCopy,
            "PrepareVectorPush(true)" => ProcedureId::PrepareVectorPush(true),
            "PrepareVectorPush(false)" => ProcedureId::PrepareVectorPush(false),
            "VectorPop(true)" => ProcedureId::VectorPop(true),
            "VectorPop(false)" => ProcedureId::VectorPop(false),
            "PrepareVectorInsert" => ProcedureId::PrepareVectorInsert,
            "VectorRemove" => ProcedureId::VectorRemove,
            "CheckMaxStackDepth" => ProcedureId::CheckMaxStackDepth,
            _ => return Err(ProcedureIdFromStrError::InvalidProcedureIdString(s.to_string())),
        };
        Ok(res)
    }
}
