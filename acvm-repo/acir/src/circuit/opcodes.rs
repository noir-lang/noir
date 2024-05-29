use super::{
    brillig::{BrilligInputs, BrilligOutputs},
    directives::Directive,
};
use crate::native_types::{Expression, Witness};
use acir_field::AcirField;
use serde::{Deserialize, Serialize};

mod black_box_function_call;
mod memory_operation;

pub use black_box_function_call::{BlackBoxFuncCall, FunctionInput};
pub use memory_operation::{BlockId, MemOp};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockType {
    Memory,
    CallData,
    ReturnData,
}

impl BlockType {
    pub fn is_databus(&self) -> bool {
        matches!(self, BlockType::CallData | BlockType::ReturnData)
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Opcode<F> {
    AssertZero(Expression<F>),
    /// Calls to "gadgets" which rely on backends implementing support for specialized constraints.
    ///
    /// Often used for exposing more efficient implementations of SNARK-unfriendly computations.  
    BlackBoxFuncCall(BlackBoxFuncCall),
    Directive(Directive<F>),
    /// Atomic operation on a block of memory
    MemoryOp {
        block_id: BlockId,
        op: MemOp<F>,
        /// Predicate of the memory operation - indicates if it should be skipped
        predicate: Option<Expression<F>>,
    },
    MemoryInit {
        block_id: BlockId,
        init: Vec<Witness>,
        block_type: BlockType,
    },
    /// Calls to unconstrained functions
    BrilligCall {
        /// Id for the function being called. It is the responsibility of the executor
        /// to fetch the appropriate Brillig bytecode from this id.
        id: u32,
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
        id: u32,
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
            Opcode::Directive(Directive::ToLeRadix { a, b, radix: _ }) => {
                write!(f, "DIR::TORADIX ")?;
                write!(
                    f,
                    // TODO (Note): this assumes that the decomposed bits have contiguous witness indices
                    // This should be the case, however, we can also have a function which checks this
                    "(_{}, [_{}..._{}] )",
                    a,
                    b.first().unwrap().witness_index(),
                    b.last().unwrap().witness_index(),
                )
            }
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
                    BlockType::CallData => write!(f, "INIT CALLDATA ")?,
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
