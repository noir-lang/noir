use super::opcodes::BlockId;
use crate::native_types::{Expression, Witness};
use brillig::Opcode as BrilligOpcode;
use serde::{Deserialize, Serialize};

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

impl<F> BrilligBytecode<F> {
    /// Returns true if the bytecode contains a foreign call
    /// whose name matches the given predicate.
    pub fn has_oracle<Fun>(&self, filter: Fun) -> bool
    where
        Fun: Fn(&str) -> bool,
    {
        self.bytecode.iter().any(|op| {
            if let BrilligOpcode::ForeignCall { function, .. } = op {
                filter(function)
            } else {
                false
            }
        })
    }
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
