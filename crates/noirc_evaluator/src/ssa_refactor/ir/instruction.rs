use super::{
    basic_block::BasicBlockId, function::FunctionId, map::Id, types::Type, value::ValueId,
};

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

impl std::fmt::Display for IntrinsicOpcodes {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("intrinsics have no opcodes yet")
    }
}

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

    /// Allocates a region of memory. Note that this is not concerned with
    /// the type of memory, the type of element is determined when loading this memory.
    ///
    /// `size` is the size of the region to be allocated by the number of FieldElements it
    /// contains. Note that non-numeric types like Functions and References are counted as 1 field
    /// each.
    Allocate { size: u32 },

    /// Loads a value from memory.
    Load { address: ValueId },

    /// Writes a value to memory.
    Store { address: ValueId, value: ValueId },
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
            Instruction::Allocate { .. } => 1,
            Instruction::Load { .. } => 1,
            Instruction::Store { .. } => 0,
        }
    }

    /// Returns the number of arguments required for a call
    pub(crate) fn num_fixed_arguments(&self) -> usize {
        // Match-all fields syntax (..) is avoided on most cases of this match to ensure that
        // if an extra argument is ever added to any of these variants, an error
        // is issued pointing to this spot to update it here as well.
        match self {
            Instruction::Binary(_) => 2,
            Instruction::Cast(_, _) => 1,
            Instruction::Not(_) => 1,
            Instruction::Truncate { value: _, bit_size: _, max_bit_size: _ } => 1,
            Instruction::Constrain(_) => 1,
            // This returns 0 as the arguments depend on the function being called
            Instruction::Call { .. } => 0,
            // This also returns 0, but we could get it a compile time,
            // since we know the function definition for the intrinsics
            Instruction::Intrinsic { .. } => 0,
            Instruction::Allocate { size: _ } => 1,
            Instruction::Load { address: _ } => 1,
            Instruction::Store { address: _, value: _ } => 2,
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
            Instruction::Allocate { .. } => vec![Type::Reference],
            Instruction::Load { .. } => vec![ctrl_typevar],
            Instruction::Store { .. } => vec![],
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
    /// If the condition is true: jump to the specified `then_destination` with `arguments`.
    /// Otherwise, jump to the specified `else_destination` with `arguments`.
    JmpIf {
        condition: ValueId,
        then_destination: BasicBlockId,
        else_destination: BasicBlockId,
        arguments: Vec<ValueId>,
    },

    /// Unconditional Jump
    ///
    /// Jumps to specified `destination` with `arguments`
    Jmp { destination: BasicBlockId, arguments: Vec<ValueId> },

    /// Return from the current function with the given return values.
    ///
    /// All finished functions should have exactly 1 return instruction.
    /// Functions with early returns should instead be structured to
    /// unconditionally jump to a single exit block with the return values
    /// as the block arguments. Then the exit block can terminate in a return
    /// instruction returning these values.
    Return { return_values: Vec<ValueId> },
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
/// Aside from the comparison operators (Eq and Lt), all operators
/// will return the same type as their operands.
/// The operand types must match for all binary operators.
/// All binary operators are also only for numeric types. To implement
/// e.g. equality for a compound type like a struct, one must add a
/// separate Eq operation for each field and combine them later with And.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) enum BinaryOp {
    /// Addition of lhs + rhs.
    Add,
    /// Subtraction of lhs - rhs.
    Sub,
    /// Multiplication of lhs * rhs.
    Mul,
    /// Division of lhs / rhs.
    Div,
    /// Modulus of lhs % rhs.
    Mod,
    /// Checks whether two types are equal.
    /// Returns true if the types were equal and
    /// false otherwise.
    Eq,
    /// Checks whether the lhs is less than the rhs.
    /// All other comparison operators should be translated
    /// to less than. For example (a > b) = (b < a) = !(a >= b) = !(b <= a).
    /// The result will always be a u1.
    Lt,
    /// Bitwise and (&)
    And,
    /// Bitiwse or (|)
    Or,
    /// Bitwise xor (^)
    Xor,
    /// Shift lhs left by rhs bits (<<)
    Shl,
    /// Shift lhs right by rhs bits (>>)
    Shr,
}

impl From<noirc_frontend::BinaryOpKind> for BinaryOp {
    fn from(value: noirc_frontend::BinaryOpKind) -> Self {
        match value {
            noirc_frontend::BinaryOpKind::Add => todo!(),
            noirc_frontend::BinaryOpKind::Subtract => todo!(),
            noirc_frontend::BinaryOpKind::Multiply => todo!(),
            noirc_frontend::BinaryOpKind::Divide => todo!(),
            noirc_frontend::BinaryOpKind::Equal => todo!(),
            noirc_frontend::BinaryOpKind::NotEqual => todo!(),
            noirc_frontend::BinaryOpKind::Less => todo!(),
            noirc_frontend::BinaryOpKind::LessEqual => todo!(),
            noirc_frontend::BinaryOpKind::Greater => todo!(),
            noirc_frontend::BinaryOpKind::GreaterEqual => todo!(),
            noirc_frontend::BinaryOpKind::And => todo!(),
            noirc_frontend::BinaryOpKind::Or => todo!(),
            noirc_frontend::BinaryOpKind::Xor => todo!(),
            noirc_frontend::BinaryOpKind::ShiftRight => todo!(),
            noirc_frontend::BinaryOpKind::ShiftLeft => todo!(),
            noirc_frontend::BinaryOpKind::Modulo => todo!(),
        }
    }
}

impl std::fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "add"),
            BinaryOp::Sub => write!(f, "sub"),
            BinaryOp::Mul => write!(f, "mul"),
            BinaryOp::Div => write!(f, "div"),
            BinaryOp::Eq => write!(f, "eq"),
            BinaryOp::Mod => write!(f, "mod"),
            BinaryOp::Lt => write!(f, "lt"),
            BinaryOp::And => write!(f, "and"),
            BinaryOp::Or => write!(f, "or"),
            BinaryOp::Xor => write!(f, "xor"),
            BinaryOp::Shl => write!(f, "shl"),
            BinaryOp::Shr => write!(f, "shr"),
        }
    }
}
