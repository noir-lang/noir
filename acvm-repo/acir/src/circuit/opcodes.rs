use super::brillig::{BrilligFunctionId, BrilligInputs, BrilligOutputs};

pub mod function_id;
pub use function_id::AcirFunctionId;

use crate::native_types::{Expression, Witness};
use acir_field::AcirField;
use serde::{Deserialize, Serialize};

mod black_box_function_call;
mod memory_operation;

pub use black_box_function_call::{
    BlackBoxFuncCall, ConstantOrWitnessEnum, FunctionInput, InvalidInputBitSize,
};
pub use memory_operation::{BlockId, MemOp};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum BlockType {
    Memory,
    CallData(u32),
    ReturnData,
}

impl BlockType {
    pub fn is_databus(&self) -> bool {
        matches!(self, BlockType::CallData(_) | BlockType::ReturnData)
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Opcode<F> {
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
    /// All black box functions take as input a tuple `(witness, num_bits)`,
    /// where `num_bits` is a constant representing the bit size of the input
    /// witness, and they have one or several witnesses as output.
    ///
    /// Some more advanced computations assume that the proving system has an
    /// 'embedded curve'. It is a curve that cycles with the main curve of the
    /// proving system, i.e the scalar field of the embedded curve is the base
    /// field of the main one, and vice-versa.
    ///
    /// Aztec's Barretenberg uses BN254 as the main curve and Grumpkin as the
    /// embedded curve.
    BlackBoxFuncCall(BlackBoxFuncCall<F>),

    /// Atomic operation on a block of memory
    ///
    /// ACIR is able to address any array of witnesses. Each array is assigned
    /// an id (BlockId) and needs to be initialized with the MemoryInit opcode.
    /// Then it is possible to read and write from/to an array by providing the
    /// index and the value we read/write as arithmetic expressions. Note that
    /// ACIR arrays all have a known fixed length (given in the MemoryInit
    /// opcode below)
    ///
    /// - predicate: an arithmetic expression that disables the execution of the
    ///   opcode when the expression evaluates to zero
    MemoryOp {
        /// identifier of the array
        block_id: BlockId,
        /// describe the memory operation to perform
        op: MemOp<F>,
        /// Predicate of the memory operation - indicates if it should be skipped
        predicate: Option<Expression<F>>,
    },

    /// Initialize an ACIR array from a vector of witnesses.
    /// - block_id: identifier of the array
    /// - init: Vector of witnesses specifying the initial value of the array
    ///
    /// There must be only one MemoryInit per block_id, and MemoryOp opcodes must
    /// come after the MemoryInit.
    MemoryInit { block_id: BlockId, init: Vec<Witness>, block_type: BlockType },

    /// Calls to unconstrained functions
    BrilligCall {
        /// Id for the function being called. It is the responsibility of the executor
        /// to fetch the appropriate Brillig bytecode from this id.
        id: BrilligFunctionId,
        /// Inputs to the function call
        inputs: Vec<BrilligInputs<F>>,
        /// Outputs to the function call
        outputs: Vec<BrilligOutputs>,
        /// Predicate of the Brillig execution - indicates if it should be skipped
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
        /// Predicate of the circuit execution - indicates if it should be skipped
        predicate: Option<Expression<F>>,
    },
}

impl<F: AcirField> std::fmt::Display for Opcode<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Opcode::AssertZero(expr) => {
                write!(f, "EXPR [ ")?;
                for i in &expr.mul_terms {
                    write!(f, "({}, _{}, _{}) ", i.0, i.1.witness_index(), i.2.witness_index())?;
                }
                for i in &expr.linear_combinations {
                    write!(f, "({}, _{}) ", i.0, i.1.witness_index())?;
                }
                write!(f, "{}", expr.q_c)?;

                write!(f, " ]")
            }

            Opcode::BlackBoxFuncCall(g) => write!(f, "{g}"),
            Opcode::MemoryOp { block_id, op, predicate } => {
                write!(f, "MEM ")?;
                if let Some(pred) = predicate {
                    writeln!(f, "PREDICATE = {pred}")?;
                }

                let is_read = op.operation.is_zero();
                let is_write = op.operation == Expression::one();
                if is_read {
                    write!(f, "(id: {}, read at: {}, value: {}) ", block_id.0, op.index, op.value)
                } else if is_write {
                    write!(f, "(id: {}, write {} at: {}) ", block_id.0, op.value, op.index)
                } else {
                    write!(f, "(id: {}, op {} at: {}) ", block_id.0, op.operation, op.index)
                }
            }
            Opcode::MemoryInit { block_id, init, block_type: databus } => {
                match databus {
                    BlockType::Memory => write!(f, "INIT ")?,
                    BlockType::CallData(id) => write!(f, "INIT CALLDATA {} ", id)?,
                    BlockType::ReturnData => write!(f, "INIT RETURNDATA ")?,
                }
                write!(f, "(id: {}, len: {}) ", block_id.0, init.len())
            }
            // We keep the display for a BrilligCall and circuit Call separate as they
            // are distinct in their functionality and we should maintain this separation for debugging.
            Opcode::BrilligCall { id, inputs, outputs, predicate } => {
                write!(f, "BRILLIG CALL func {}: ", id)?;
                if let Some(pred) = predicate {
                    writeln!(f, "PREDICATE = {pred}")?;
                }
                write!(f, "inputs: {:?}, ", inputs)?;
                write!(f, "outputs: {:?}", outputs)
            }
            Opcode::Call { id, inputs, outputs, predicate } => {
                write!(f, "CALL func {}: ", id)?;
                if let Some(pred) = predicate {
                    writeln!(f, "PREDICATE = {pred}")?;
                }
                write!(f, "inputs: {:?}, ", inputs)?;
                write!(f, "outputs: {:?}", outputs)
            }
        }
    }
}

impl<F: AcirField> std::fmt::Debug for Opcode<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}
