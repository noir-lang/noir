use super::{brillig::Brillig, directives::Directive};
use crate::native_types::{Expression, Witness};
use serde::{Deserialize, Serialize};

mod black_box_function_call;
mod memory_operation;

pub use black_box_function_call::{BlackBoxFuncCall, FunctionInput};
pub use memory_operation::{BlockId, MemOp};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Opcode {
    AssertZero(Expression),
    /// Calls to "gadgets" which rely on backends implementing support for specialized constraints.
    ///
    /// Often used for exposing more efficient implementations of SNARK-unfriendly computations.  
    BlackBoxFuncCall(BlackBoxFuncCall),
    Directive(Directive),
    Brillig(Brillig),
    /// Atomic operation on a block of memory
    MemoryOp {
        block_id: BlockId,
        op: MemOp,
        /// Predicate of the memory operation - indicates if it should be skipped
        predicate: Option<Expression>,
    },
    MemoryInit {
        block_id: BlockId,
        init: Vec<Witness>,
    },
}

impl std::fmt::Display for Opcode {
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
            Opcode::Directive(Directive::PermutationSort { inputs: a, tuple, bits, sort_by }) => {
                write!(f, "DIR::PERMUTATIONSORT ")?;
                write!(
                    f,
                    "(permutation size: {} {}-tuples, sort_by: {:#?}, bits: [_{}..._{}]))",
                    a.len(),
                    tuple,
                    sort_by,
                    // (Note): the bits do not have contiguous index but there are too many for display
                    bits.first().unwrap().witness_index(),
                    bits.last().unwrap().witness_index(),
                )
            }

            Opcode::Brillig(brillig) => {
                write!(f, "BRILLIG: ")?;
                writeln!(f, "inputs: {:?}", brillig.inputs)?;
                writeln!(f, "outputs: {:?}", brillig.outputs)?;
                writeln!(f, "{:?}", brillig.bytecode)
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
            Opcode::MemoryInit { block_id, init } => {
                write!(f, "INIT ")?;
                write!(f, "(id: {}, len: {}) ", block_id.0, init.len())
            }
        }
    }
}

impl std::fmt::Debug for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}
