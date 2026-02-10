//! This module contains [Brillig][brillig] structures for integration within an ACIR circuit.
//!
//! [Brillig][brillig] is used in ACIR as a hint for the solver when executing the circuit.
//! Executing Brillig does not generate any constraints and is the result of the compilation of an unconstrained function.
//!
//! Let's see an example with euclidean division.
//! The normal way to compute `a/b`, where `a` and `b` are 8-bits integers, is to
//! implement the Euclidean algorithm which computes in a loop (or recursively)
//! modulus of the kind 'a mod b'. Doing this computation requires a lot of steps to
//! be properly implemented in ACIR, especially the loop with a condition. However,
//! euclidean division can be easily constrained with one assert-zero opcode:
//! `a = bq+r`, assuming `q` is 8 bits and `r<b`. Since these assumptions can easily
//! written with a few range opcodes, euclidean division can in fact be implemented
//! with a small number of opcodes.
//!
//! However, in order to write these opcodes we need the result of the division
//! which are the witness `q` and `r`. But from the constraint `a=bq+r`, how can the
//! solver figure out how to solve `q` and `r` with only one equation? This is where
//! brillig/unconstrained function come into action. We simply define a function that
//! performs the usual Euclidean algorithm to compute `q` and `r` from `a` and `b`.
//! Since executing Brillig does not generate constraints, it is not meaningful to the
//! proving system but simply used by the solver to compute the values of `q` and
//! `r`. The output witnesses `q` and `r` are then expected to be used by the proving system.
//!
//! In summary, executing Brillig will perform the computation defined by its
//! bytecode, on the provided inputs, and assign the result to the output witnesses,
//! without adding any constraints.

use super::opcodes::BlockId;
use crate::native_types::{Expression, Witness};
use acir_field::AcirField;
use brillig::Opcode as BrilligOpcode;
use serde::{Deserialize, Serialize};

/// Inputs for the Brillig VM. These are the initial inputs
/// that the Brillig VM will use to start.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Hash)]
#[cfg_attr(feature = "arb", derive(proptest_derive::Arbitrary))]
pub enum BrilligInputs<F> {
    Single(Expression<F>),
    Array(Vec<Expression<F>>),
    MemoryArray(BlockId),
}

impl<F: AcirField> std::fmt::Display for BrilligInputs<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrilligInputs::Single(expr) => expr.fmt(f),
            BrilligInputs::Array(exprs) => {
                let joined = exprs.iter().map(|e| format!("{e}")).collect::<Vec<_>>().join(", ");
                write!(f, "[{joined}]")
            }
            BrilligInputs::MemoryArray(block_id) => block_id.fmt(f),
        }
    }
}

/// Outputs for the Brillig VM. Once the VM has completed
/// execution, this will be the object that is returned.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Hash)]
#[cfg_attr(feature = "arb", derive(proptest_derive::Arbitrary))]
pub enum BrilligOutputs {
    Simple(Witness),
    Array(Vec<Witness>),
}

impl std::fmt::Display for BrilligOutputs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrilligOutputs::Simple(witness) => write!(f, "{witness}"),
            BrilligOutputs::Array(witnesses) => {
                let joined =
                    witnesses.iter().map(|w| format!("{w}")).collect::<Vec<_>>().join(", ");
                write!(f, "[{joined}]")
            }
        }
    }
}

/// This is purely a wrapper struct around a list of Brillig opcode's which represents
/// a full Brillig function to be executed by the Brillig VM.
/// This is stored separately on a program and accessed through a [BrilligFunctionId].
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Default, Debug, Hash)]
#[cfg_attr(feature = "arb", derive(proptest_derive::Arbitrary))]
pub struct BrilligBytecode<F> {
    #[serde(default)] // For backwards compatibility
    pub function_name: String,
    pub bytecode: Vec<BrilligOpcode<F>>,
}

/// Id for the function being called.
/// Indexes into the table of Brillig function's specified in a [program][super::Program]
#[derive(
    Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash, Copy, Default, PartialOrd, Ord,
)]
#[cfg_attr(feature = "arb", derive(proptest_derive::Arbitrary))]
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
