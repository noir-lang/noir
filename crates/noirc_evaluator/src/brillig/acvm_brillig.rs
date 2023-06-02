use acvm::FieldElement;
//THIS IS A TEMPORARY FILE, all warnings are disabled.
#[allow(clippy::all, unreachable_code, unreachable_pub, rustdoc::all, dead_code)]

pub type Label = usize;
#[allow(clippy::all, unreachable_code, unreachable_pub, rustdoc::all)]
pub type BrilligOpcode = Opcode;
#[allow(clippy::all, unreachable_code, unreachable_pub, rustdoc::all, dead_code)]
pub type BrilligType = Typ;
#[allow(clippy::all, unreachable_code, unreachable_pub, rustdoc::all, dead_code)]
pub type BrilligValue = Value;

#[allow(clippy::all, unreachable_code, unreachable_pub, rustdoc::all, dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum RegisterValueOrArray {
    RegisterIndex(RegisterIndex),
    HeapArray(RegisterIndex, usize),
}
#[allow(clippy::all, unreachable_code, unreachable_pub, rustdoc::all, dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
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
        /// Destination register (may be a memory pointer).
        destination: RegisterValueOrArray,
        /// Input register (may be a memory pointer).
        input: RegisterValueOrArray,
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
    /// Used to denote execution failure
    Trap,
    /// Stop execution
    Stop,
}

#[allow(clippy::all, unreachable_code, unreachable_pub, rustdoc::all, dead_code)]
/// Binary fixed-length field expressions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryFieldOp {
    Add,
    Sub,
    Mul,
    Div,
    Cmp(Comparison),
}
#[allow(clippy::all, unreachable_code, unreachable_pub, rustdoc::all, dead_code)]
/// Binary fixed-length integer expressions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryIntOp {
    Add,
    Sub,
    Mul,
    SignedDiv,
    UnsignedDiv,
    Cmp(Comparison),
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
#[allow(clippy::all, unreachable_code, unreachable_pub, rustdoc::all, dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comparison {
    /// (==) equal
    Eq,
    /// (<) Field less than
    Lt,
    /// (<=) field less or equal
    Lte,
}

#[allow(clippy::all, unreachable_code, unreachable_pub, rustdoc::all, dead_code)]
/// `RegisterIndex` refers to the index of a register in the VM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegisterIndex(pub usize);
#[allow(clippy::all, unreachable_code, unreachable_pub, rustdoc::all, dead_code)]
/// Types of values allowed in the VM
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Typ {
    Field,
    Unsigned { bit_size: u32 },
    Signed { bit_size: u32 },
}
#[allow(clippy::all, unreachable_code, unreachable_pub, rustdoc::all, dead_code)]
/// `Value` represents the base descriptor for a value in the VM.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Value {
    pub inner: FieldElement,
}
