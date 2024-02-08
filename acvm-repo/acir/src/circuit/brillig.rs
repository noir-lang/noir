use super::opcodes::BlockId;
use crate::native_types::{Expression, Witness};
use brillig::Opcode as BrilligOpcode;
use serde::{Deserialize, Serialize};

/// Inputs for the Brillig VM. These are the initial inputs
/// that the Brillig VM will use to start.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum BrilligInputs {
    Single(Expression),
    Array(Vec<Expression>),
    MemoryArray(BlockId),
}

/// Outputs for the Brillig VM. Once the VM has completed
/// execution, this will be the object that is returned.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum BrilligOutputs {
    Simple(Witness),
    Array(Vec<Witness>),
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct Brillig {
    pub inputs: Vec<BrilligInputs>,
    pub outputs: Vec<BrilligOutputs>,
    /// The Brillig VM bytecode to be executed by this ACIR opcode.
    pub bytecode: Vec<BrilligOpcode>,
    /// Predicate of the Brillig execution - indicates if it should be skipped
    pub predicate: Option<Expression>,
}
