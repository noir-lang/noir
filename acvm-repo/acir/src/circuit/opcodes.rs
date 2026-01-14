//! ACIR opcodes
//!
//! This module defines the core set opcodes used in ACIR.
use super::brillig::{BrilligFunctionId, BrilligInputs, BrilligOutputs};

pub mod function_id;
pub use function_id::AcirFunctionId;

use crate::{
    circuit::PublicInputs,
    native_types::{Expression, Witness, display_expression},
};
use acir_field::AcirField;
use serde::{Deserialize, Serialize};

mod black_box_function_call;
mod memory_operation;

pub use black_box_function_call::{BlackBoxFuncCall, FunctionInput, InvalidInputBitSize};
pub use memory_operation::{BlockId, MemOp};

/// Type for a memory block
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[cfg_attr(feature = "arb", derive(proptest_derive::Arbitrary))]
pub enum BlockType {
    /// The default type of memory block.
    /// Virtually all user memory blocks are expected to be of this type
    /// unless the backend wishes to expose special handling for call/return data.
    Memory,
    /// Indicate to the backend that this memory comes from a circuit's inputs.
    ///
    /// This is most useful for schemes which require passing a lot of circuit inputs
    /// through multiple circuits (such as in a recursive proof scheme).
    /// Stores a constant identifier to distinguish between multiple calldata inputs.
    CallData(u32),
    /// Similar to calldata except it states that this memory is returned in the circuit outputs.
    ReturnData,
}

impl BlockType {
    pub fn is_databus(&self) -> bool {
        matches!(self, BlockType::CallData(_) | BlockType::ReturnData)
    }
}

/// Defines an operation within an ACIR circuit
///
/// Expects a type parameter `F` which implements [AcirField].
#[allow(clippy::large_enum_variant)]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[cfg_attr(feature = "arb", derive(proptest_derive::Arbitrary))]
pub enum Opcode<F: AcirField> {
    /// An `AssertZero` opcode adds the constraint that `P(w) = 0`, where
    /// `w=(w_1,..w_n)` is a tuple of `n` witnesses, and `P` is a multi-variate
    /// polynomial of total degree at most `2`.
    ///
    /// The coefficients `{q_M}_{i,j}, q_i,q_c` of the polynomial are known
    /// values which define the opcode.
    ///
    /// A general expression of assert-zero opcode is the following:
    /// ```text
    /// \sum_{i,j} {q_M}_{i,j}w_iw_j + \sum_i q_iw_i +q_c = 0
    /// ```
    ///
    /// An assert-zero opcode can be used to:
    /// - **express a constraint** on witnesses; for instance to express that a
    ///   witness `w` is a boolean, you can add the opcode: `w*w-w=0`
    /// - or, to **compute the value** of an arithmetic operation of some inputs.
    ///
    /// For instance, to multiply two witnesses `x` and `y`, you would use the
    /// opcode `z-x*y=0`, which would constrain `z` to be `x*y`.
    ///
    /// The solver expects that at most one witness is not known when executing the opcode.
    AssertZero(Expression<F>),

    /// Calls to "gadgets" which rely on backends implementing support for
    /// specialized constraints.
    ///
    /// Often used for exposing more efficient implementations of
    /// SNARK-unfriendly computations.
    ///
    /// All black box function inputs are specified as [FunctionInput],
    /// and they have one or several witnesses as output.
    ///
    /// Some more advanced computations assume that the proving system has an
    /// 'embedded curve'. It is a curve that cycles with the main curve of the
    /// proving system, i.e the scalar field of the embedded curve is the base
    /// field of the main one, and vice-versa.
    /// e.g. Aztec's Barretenberg uses BN254 as the main curve and Grumpkin as the
    /// embedded curve.
    BlackBoxFuncCall(BlackBoxFuncCall<F>),

    /// Atomic operation on a block of memory
    ///
    /// ACIR is able to address any array of witnesses. Each array is assigned
    /// an id ([BlockId]) and needs to be initialized with the [Opcode::MemoryInit] opcode.
    /// Then it is possible to read and write from/to an array by providing the
    /// index and the value we read/write as arithmetic expressions. Note that
    /// ACIR arrays all have a known fixed length (given in the [Opcode::MemoryInit]
    /// opcode below)
    MemoryOp {
        /// Identifier of the array
        block_id: BlockId,
        /// Describe the memory operation to perform
        op: MemOp,
    },

    /// Initialize an ACIR array from a vector of witnesses.
    ///
    /// There must be only one MemoryInit per block_id, and MemoryOp opcodes must
    /// come after the MemoryInit.
    MemoryInit {
        /// Identifier of the array
        block_id: BlockId,
        /// Vector of witnesses specifying the initial value of the array
        init: Vec<Witness>,
        /// Specify what type of memory we should initialize
        block_type: BlockType,
    },

    /// Calls to unconstrained functions. Unconstrained functions are constructed with [Brillig][super::brillig].
    BrilligCall {
        /// Id for the function being called. It is the responsibility of the executor
        /// to fetch the appropriate Brillig bytecode from this id.
        id: BrilligFunctionId,
        /// Inputs to the function call
        inputs: Vec<BrilligInputs<F>>,
        /// Outputs to the function call
        outputs: Vec<BrilligOutputs>,
        /// Predicate of the Brillig execution - when the predicate evaluates to 0, execution is skipped.
        /// When the predicate evaluates to 1 or is None, execution proceeds.
        predicate: Option<Expression<F>>,
    },

    /// Calls to functions represented as a separate circuit. A call opcode allows us
    /// to build a call stack when executing the outer-most circuit.
    Call {
        /// Id for the function being called. It is the responsibility of the executor
        /// to fetch the appropriate circuit from this id.
        id: AcirFunctionId,
        /// Inputs to the function call
        inputs: Vec<Witness>,
        /// Outputs of the function call
        outputs: Vec<Witness>,
        /// Predicate of the circuit execution - when the predicate evaluates to 0, execution is skipped.
        /// When the predicate evaluates to 1 or is None, execution proceeds.
        predicate: Option<Expression<F>>,
    },
}

impl<F: AcirField> std::fmt::Display for Opcode<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        display_opcode(self, None, f)
    }
}

impl<F: AcirField> std::fmt::Debug for Opcode<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

/// Displays an opcode, optionally using the provided return values to prefer displaying
/// `ASSERT return_value = ...` when possible.
pub(super) fn display_opcode<F: AcirField>(
    opcode: &Opcode<F>,
    return_values: Option<&PublicInputs>,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    match opcode {
        Opcode::AssertZero(expr) => {
            write!(f, "ASSERT ")?;
            display_expression(expr, true, return_values, f)
        }
        Opcode::BlackBoxFuncCall(g) => std::fmt::Display::fmt(&g, f),
        Opcode::MemoryOp { block_id, op } => {
            let is_write = op.operation;
            if is_write {
                write!(f, "WRITE b{}[{}] = {}", block_id.0, op.index, op.value)
            } else {
                write!(f, "READ {} = b{}[{}]", op.value, block_id.0, op.index)
            }
        }
        Opcode::MemoryInit { block_id, init, block_type: databus } => {
            match databus {
                BlockType::Memory => write!(f, "INIT ")?,
                BlockType::CallData(id) => write!(f, "INIT CALLDATA {id} ")?,
                BlockType::ReturnData => write!(f, "INIT RETURNDATA ")?,
            }
            let witnesses = init.iter().map(|w| format!("{w}")).collect::<Vec<String>>().join(", ");
            write!(f, "b{} = [{witnesses}]", block_id.0)
        }
        // We keep the display for a BrilligCall and circuit Call separate as they
        // are distinct in their functionality and we should maintain this separation for debugging.
        Opcode::BrilligCall { id, inputs, outputs, predicate } => {
            write!(f, "BRILLIG CALL func: {id}, ")?;
            if let Some(pred) = predicate {
                write!(f, "predicate: {pred}, ")?;
            }

            let inputs =
                inputs.iter().map(|input| format!("{input}")).collect::<Vec<String>>().join(", ");
            let outputs = outputs
                .iter()
                .map(|output| format!("{output}"))
                .collect::<Vec<String>>()
                .join(", ");

            write!(f, "inputs: [{inputs}], ")?;
            write!(f, "outputs: [{outputs}]")
        }
        Opcode::Call { id, inputs, outputs, predicate } => {
            write!(f, "CALL func: {id}, ")?;
            if let Some(pred) = predicate {
                write!(f, "predicate: {pred}, ")?;
            }
            let inputs = inputs.iter().map(|w| format!("{w}")).collect::<Vec<String>>().join(", ");
            let outputs =
                outputs.iter().map(|w| format!("{w}")).collect::<Vec<String>>().join(", ");

            write!(f, "inputs: [{inputs}], ")?;
            write!(f, "outputs: [{outputs}]")
        }
    }
}

#[cfg(test)]
mod tests {
    use acir_field::FieldElement;

    use crate::{
        circuit::opcodes::{BlackBoxFuncCall, BlockId, BlockType, FunctionInput},
        native_types::{Expression, Witness},
    };

    use super::Opcode;

    #[test]
    fn mem_init_display_snapshot() {
        let mem_init: Opcode<FieldElement> = Opcode::MemoryInit {
            block_id: BlockId(42),
            init: (0..10u32).map(Witness).collect(),
            block_type: BlockType::Memory,
        };

        insta::assert_snapshot!(
            mem_init.to_string(),
            @"INIT b42 = [w0, w1, w2, w3, w4, w5, w6, w7, w8, w9]"
        );
    }

    #[test]
    fn blackbox_snapshot() {
        let xor: Opcode<FieldElement> = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::XOR {
            lhs: FunctionInput::Witness(0.into()),
            rhs: FunctionInput::Witness(1.into()),
            num_bits: 32,
            output: Witness(3),
        });

        insta::assert_snapshot!(
            xor.to_string(),
            @"BLACKBOX::XOR lhs: w0, rhs: w1, output: w3, bits: 32"
        );
    }

    #[test]
    fn range_display_snapshot() {
        let range: Opcode<FieldElement> = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
            input: FunctionInput::Witness(0.into()),
            num_bits: 32,
        });

        insta::assert_snapshot!(
            range.to_string(),
            @"BLACKBOX::RANGE input: w0, bits: 32"
        );
    }

    #[test]
    fn display_zero() {
        let zero = Opcode::AssertZero(Expression::<FieldElement>::default());
        assert_eq!(zero.to_string(), "ASSERT 0 = 0");
    }
}
