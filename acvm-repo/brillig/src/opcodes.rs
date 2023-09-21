use crate::{black_box::BlackBoxOp, Value};
use serde::{Deserialize, Serialize};

pub type Label = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RegisterIndex(pub usize);

/// `RegisterIndex` refers to the index in VM register space.
impl RegisterIndex {
    pub fn to_usize(self) -> usize {
        self.0
    }
}

impl From<usize> for RegisterIndex {
    fn from(value: usize) -> Self {
        RegisterIndex(value)
    }
}

/// A fixed-sized array starting from a Brillig register memory location.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub struct HeapArray {
    pub pointer: RegisterIndex,
    pub size: usize,
}

/// A register-sized vector passed starting from a Brillig register memory location and with a register-held size
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub struct HeapVector {
    pub pointer: RegisterIndex,
    pub size: RegisterIndex,
}

/// Lays out various ways an external foreign call's input and output data may be interpreted inside Brillig.
/// This data can either be an individual register value or memory.
///
/// While we are usually agnostic to how memory is passed within Brillig,
/// this needs to be encoded somehow when dealing with an external system.
/// For simplicity, the extra type information is given right in the ForeignCall instructions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub enum RegisterOrMemory {
    /// A single register value passed to or from an external call
    /// It is an 'immediate' value - used without dereferencing memory.
    /// For a foreign call input, the value is read directly from the register.
    /// For a foreign call output, the value is written directly to the register.
    RegisterIndex(RegisterIndex),
    /// An array passed to or from an external call
    /// In the case of a foreign call input, the array is read from this Brillig memory location + usize more cells.
    /// In the case of a foreign call output, the array is written to this Brillig memory location with the usize being here just as a sanity check for the size write.
    HeapArray(HeapArray),
    /// A vector passed to or from an external call
    /// In the case of a foreign call input, the vector is read from this Brillig memory location + as many cells as the 2nd register indicates.
    /// In the case of a foreign call output, the vector is written to this Brillig memory location and as 'size' cells, with size being stored in the second register.
    HeapVector(HeapVector),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Opcode {
    /// Takes the fields in registers `lhs` and `rhs`
    /// Performs the specified binary operation
    /// and stores the value in the `result` register.  
    BinaryFieldOp {
        destination: RegisterIndex,
        op: BinaryFieldOp,
        lhs: RegisterIndex,
        rhs: RegisterIndex,
    },
    /// Takes the `bit_size` size integers in registers `lhs` and `rhs`
    /// Performs the specified binary operation
    /// and stores the value in the `result` register.  
    BinaryIntOp {
        destination: RegisterIndex,
        op: BinaryIntOp,
        bit_size: u32,
        lhs: RegisterIndex,
        rhs: RegisterIndex,
    },
    JumpIfNot {
        condition: RegisterIndex,
        location: Label,
    },
    /// Sets the program counter to the value located at `destination`
    /// If the value at `condition` is non-zero
    JumpIf {
        condition: RegisterIndex,
        location: Label,
    },
    /// Sets the program counter to the label.
    Jump {
        location: Label,
    },
    /// We don't support dynamic jumps or calls
    /// See https://github.com/ethereum/aleth/issues/3404 for reasoning
    Call {
        location: Label,
    },
    Const {
        destination: RegisterIndex,
        value: Value,
    },
    Return,
    /// Used to get data from an outside source.
    /// Also referred to as an Oracle. However, we don't use that name as
    /// this is intended for things like state tree reads, and shouldn't be confused
    /// with e.g. blockchain price oracles.
    ForeignCall {
        /// Interpreted by caller context, ie this will have different meanings depending on
        /// who the caller is.
        function: String,
        /// Destination registers (may be single values or memory pointers).
        destinations: Vec<RegisterOrMemory>,
        /// Input registers (may be single values or memory pointers).
        inputs: Vec<RegisterOrMemory>,
    },
    Mov {
        destination: RegisterIndex,
        source: RegisterIndex,
    },
    Load {
        destination: RegisterIndex,
        source_pointer: RegisterIndex,
    },
    Store {
        destination_pointer: RegisterIndex,
        source: RegisterIndex,
    },
    BlackBox(BlackBoxOp),
    /// Used to denote execution failure
    Trap,
    /// Stop execution
    Stop,
}

impl Opcode {
    pub fn name(&self) -> &'static str {
        match self {
            Opcode::BinaryFieldOp { .. } => "binary_field_op",
            Opcode::BinaryIntOp { .. } => "binary_int_op",
            Opcode::JumpIfNot { .. } => "jmp_if_not",
            Opcode::JumpIf { .. } => "jmp_if",
            Opcode::Jump { .. } => "jmp",
            Opcode::Call { .. } => "call",
            Opcode::Const { .. } => "const",
            Opcode::Return => "return",
            Opcode::ForeignCall { .. } => "foreign_call",
            Opcode::Mov { .. } => "mov",
            Opcode::Load { .. } => "load",
            Opcode::Store { .. } => "store",
            Opcode::BlackBox(_) => "black_box",
            Opcode::Trap => "trap",
            Opcode::Stop => "stop",
        }
    }
}

/// Binary fixed-length field expressions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryFieldOp {
    Add,
    Sub,
    Mul,
    Div,
    /// (==) equal
    Equals,
}

/// Binary fixed-length integer expressions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryIntOp {
    Add,
    Sub,
    Mul,
    SignedDiv,
    UnsignedDiv,
    /// (==) equal
    Equals,
    /// (<) Field less than
    LessThan,
    /// (<=) field less or equal
    LessThanEquals,
    /// (&) Bitwise AND
    And,
    /// (|) Bitwise OR
    Or,
    /// (^) Bitwise XOR
    Xor,
    /// (<<) Shift left
    Shl,
    /// (>>) Shift right
    Shr,
}
