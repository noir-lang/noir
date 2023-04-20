use acvm::FieldElement;

use super::{
    function::FunctionId,
    map::{Id, SparseMap},
    types::Type,
    value::ValueId,
};
use crate::ssa_refactor::basic_block::{BasicBlockId, BlockArguments};

// Container for all Instructions, per-function
pub(crate) type Instructions = SparseMap<Instruction>;

/// Reference to an instruction
pub(crate) type InstructionId = Id<Instruction>;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
/// These are similar to built-ins in other languages.
/// These can be classified under two categories:
/// - Opcodes which the IR knows the target machine has
/// special support for. (LowLevel)
/// - Opcodes which have no function definition in the
/// source code and must be processed by the IR. An example
/// of this is println.
pub(crate) struct IntrinsicOpcodes;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
/// Instructions are used to perform tasks.
/// The instructions that the IR is able to specify are listed below.
pub(crate) enum Instruction {
    /// Binary Operations like +, -, *, /, ==, !=
    Binary(Binary),

    /// Converts `Value` into Typ
    Cast(ValueId, Type),

    /// Computes a bit wise not
    Not(ValueId),

    /// Truncates `value` to `bit_size`
    Truncate { value: ValueId, bit_size: u32, max_bit_size: u32 },

    /// Constrains a value to be equal to true
    Constrain(ValueId),

    /// Performs a function call with a list of its arguments.
    Call { func: FunctionId, arguments: Vec<ValueId> },
    /// Performs a call to an intrinsic function and stores the
    /// results in `return_arguments`.
    Intrinsic { func: IntrinsicOpcodes, arguments: Vec<ValueId> },

    /// Loads a value from memory.
    Load(ValueId),

    /// Writes a value to memory.
    Store { destination: ValueId, value: ValueId },

    /// Stores an Immediate value
    Immediate { value: FieldElement },
}

impl Instruction {
    /// Returns the number of results that this instruction
    /// produces.
    pub(crate) fn num_fixed_results(&self) -> usize {
        match self {
            Instruction::Binary(_) => 1,
            Instruction::Cast(..) => 0,
            Instruction::Not(_) => 1,
            Instruction::Truncate { .. } => 1,
            Instruction::Constrain(_) => 0,
            // This returns 0 as the result depends on the function being called
            Instruction::Call { .. } => 0,
            // This also returns 0, but we could get it a compile time,
            // since we know the signatures for the intrinsics
            Instruction::Intrinsic { .. } => 0,
            Instruction::Load(_) => 1,
            Instruction::Store { .. } => 0,
            Instruction::Immediate { .. } => 1,
        }
    }

    /// Returns the number of arguments required for a call
    pub(crate) fn num_fixed_arguments(&self) -> usize {
        match self {
            Instruction::Binary(_) => 2,
            Instruction::Cast(..) => 1,
            Instruction::Not(_) => 1,
            Instruction::Truncate { .. } => 1,
            Instruction::Constrain(_) => 1,
            // This returns 0 as the arguments depend on the function being called
            Instruction::Call { .. } => 0,
            // This also returns 0, but we could get it a compile time,
            // since we know the function definition for the intrinsics
            Instruction::Intrinsic { .. } => 0,
            Instruction::Load(_) => 1,
            Instruction::Store { .. } => 2,
            Instruction::Immediate { .. } => 0,
        }
    }

    /// Returns the types that this instruction will return.
    pub(crate) fn return_types(&self, ctrl_typevar: Type) -> Vec<Type> {
        match self {
            Instruction::Binary(_) => vec![ctrl_typevar],
            Instruction::Cast(_, typ) => vec![*typ],
            Instruction::Not(_) => vec![ctrl_typevar],
            Instruction::Truncate { .. } => vec![ctrl_typevar],
            Instruction::Constrain(_) => vec![],
            Instruction::Call { .. } => vec![],
            Instruction::Intrinsic { .. } => vec![],
            Instruction::Load(_) => vec![ctrl_typevar],
            Instruction::Store { .. } => vec![],
            Instruction::Immediate { .. } => vec![],
        }
    }
}

/// These are operations which can exit a basic block
/// ie control flow type operations
///
/// Since our IR needs to be in SSA form, it makes sense
/// to split up instructions like this, as we are sure that these instructions
/// will not be in the list of instructions for a basic block.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) enum TerminatorInstruction {
    /// Control flow
    ///
    /// Jump If
    ///
    /// Jumps to the specified `destination` with
    /// arguments, if the condition
    /// if the condition is true.
    JmpIf { condition: ValueId, destination: BasicBlockId, arguments: BlockArguments },
    /// Unconditional Jump
    ///
    /// Jumps to specified `destination` with `arguments`
    Jmp { destination: BasicBlockId, arguments: BlockArguments },
}

/// A binary instruction in the IR.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) struct Binary {
    /// Left hand side of the binary operation
    pub(crate) lhs: ValueId,
    /// Right hand side of the binary operation
    pub(crate) rhs: ValueId,
    /// The binary operation to apply
    pub(crate) operator: BinaryOp,
}

/// Binary Operations allowed in the IR.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) enum BinaryOp {
    /// Addition of two types.
    /// The result will have the same type as
    /// the operands.
    Add,
    /// Subtraction of two types.
    /// The result will have the same type as
    /// the operands.
    Sub,
    /// Multiplication of two types.
    /// The result will have the same type as
    /// the operands.
    Mul,
    /// Division of two types.
    /// The result will have the same type as
    /// the operands.
    Div,
    /// Checks whether two types are equal.
    /// Returns true if the types were equal and
    /// false otherwise.
    Eq,
    /// Checks whether two types are equal.
    /// Returns true if the types were not equal and
    /// false otherwise.
    Ne,
}

#[test]
fn smoke_instructions_map_duplicate() {
    let id = Id::test_new(0);

    let ins = Instruction::Not(id);
    let same_ins = Instruction::Not(id);

    let mut ins_map = Instructions::default();

    // Document what happens when we insert the same instruction twice
    let id = ins_map.push(ins);
    let id_same_ins = ins_map.push(same_ins);

    // The map is quite naive and does not check if the instruction has ben inserted
    // before. We simply assign a different Id.
    assert_ne!(id, id_same_ins)
}

#[test]
fn num_instructions_smoke() {
    let n = 100;

    let mut ins_map = Instructions::default();
    for i in 0..n {
        let ins = Instruction::Not(Id::test_new(i));
        ins_map.push(ins);
    }

    assert_eq!(n, ins_map.len())
}
