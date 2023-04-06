use crate::ssa_refactor::basic_block::{BasicBlockId, BlockArguments};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
/// All types representable in the IR.
pub struct Typ;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
/// Value is the most basic type allowed in the IR.
/// Transition Note: This is similar to `NodeId` in our previous IR.
pub struct Value;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Register;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
/// These are similar to built-ins in other languages.
/// These can be classified under two categories:
/// - Opcodes which the IR knows the target machine has
/// special support for. (LowLevel)
/// - Opcodes which have no function definition in the
/// source code and must be processed by the IR. An example
/// of this is println.
pub struct IntrinsicOpcodes;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
/// Instructions are used to perform tasks.
/// The instructions that the IR is able to specify are listed below.
pub(crate) enum Instruction {
    // Binary Operations
    Binary(Binary),

    // Unary Operations
    //
    /// Converts `Value` into Typ
    Cast(Value, Typ),

    /// Computes a bit wise not
    Not(Value),

    /// Truncates `value` to `bit_size`
    Truncate {
        value: Value,
        bit_size: u32,
        max_bit_size: u32,
    },

    /// Constrains a value to be equal to true
    Constrain(Value),

    /// Performs a function call and stores the results in
    /// the `return_arguments`
    Call {
        func: Value,
        arguments: Vec<Value>,
        return_arguments: Vec<Value>,
    },
    /// Performs a call to an intrinsic function and stores the
    /// results in `return_arguments`.
    Intrinsic {
        func: IntrinsicOpcodes,
        return_arguments: Vec<Value>,
    },

    /// Loads a value from memory.
    Load(Value),

    /// Writes a value to memory.
    Store {
        destination: Value,
        value: Value,
    },
}

/// These are operations which can exit a basic block
/// ie control flow type operations
///
/// Since our IR needs to be in SSA form, it makes sense
/// to split up instructions like this, as we are sure that these instructions
/// will not be in the list of instructions for a basic block
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) enum TerminatorInstruction {
    /// Control flow
    ///
    /// Jump If
    ///
    /// Jumps to the specified `destination` with
    /// arguments, if the condition
    /// if the condition is true.
    JmpIf { condition: Value, destination: BasicBlockId, arguments: BlockArguments },
    /// Unconditional Jump
    ///
    /// Jumps to specified `destination` with `arguments`
    Jmp { destination: BasicBlockId, arguments: BlockArguments },
}

/// A binary instruction in the IR.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) struct Binary {
    /// Left hand side of the binary operation
    pub(crate) lhs: Value,
    /// Right hand side of the binary operation
    pub(crate) rhs: Value,
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
