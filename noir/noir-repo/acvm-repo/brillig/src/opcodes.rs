use crate::black_box::BlackBoxOp;
use acir_field::AcirField;
use serde::{Deserialize, Serialize};

pub type Label = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MemoryAddress(pub usize);

/// `MemoryAddress` refers to the index in VM memory.
impl MemoryAddress {
    pub fn to_usize(self) -> usize {
        self.0
    }
}

impl From<usize> for MemoryAddress {
    fn from(value: usize) -> Self {
        MemoryAddress(value)
    }
}

/// Describes the memory layout for an array/vector element
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum HeapValueType {
    // A single field element is enough to represent the value with a given bit size
    Simple(BitSize),
    // The value read should be interpreted as a pointer to a heap array, which
    // consists of a pointer to a slice of memory of size elements, and a
    // reference count
    Array { value_types: Vec<HeapValueType>, size: usize },
    // The value read should be interpreted as a pointer to a heap vector, which
    // consists of a pointer to a slice of memory, a number of elements in that
    // slice, and a reference count
    Vector { value_types: Vec<HeapValueType> },
}

impl HeapValueType {
    pub fn all_simple(types: &[HeapValueType]) -> bool {
        types.iter().all(|typ| matches!(typ, HeapValueType::Simple(_)))
    }

    pub fn field() -> HeapValueType {
        HeapValueType::Simple(BitSize::Field)
    }
}

/// A fixed-sized array starting from a Brillig memory location.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub struct HeapArray {
    pub pointer: MemoryAddress,
    pub size: usize,
}

impl Default for HeapArray {
    fn default() -> Self {
        Self { pointer: MemoryAddress(0), size: 0 }
    }
}

/// A memory-sized vector passed starting from a Brillig memory location and with a memory-held size
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub struct HeapVector {
    pub pointer: MemoryAddress,
    pub size: MemoryAddress,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, PartialOrd, Ord)]
pub enum IntegerBitSize {
    U0, // Uninitialized
    U1,
    U8,
    U16,
    U32,
    U64,
    U128,
}

impl From<IntegerBitSize> for u32 {
    fn from(bit_size: IntegerBitSize) -> u32 {
        match bit_size {
            IntegerBitSize::U0 => 0,
            IntegerBitSize::U1 => 1,
            IntegerBitSize::U8 => 8,
            IntegerBitSize::U16 => 16,
            IntegerBitSize::U32 => 32,
            IntegerBitSize::U64 => 64,
            IntegerBitSize::U128 => 128,
        }
    }
}

impl TryFrom<u32> for IntegerBitSize {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(IntegerBitSize::U0),
            1 => Ok(IntegerBitSize::U1),
            8 => Ok(IntegerBitSize::U8),
            16 => Ok(IntegerBitSize::U16),
            32 => Ok(IntegerBitSize::U32),
            64 => Ok(IntegerBitSize::U64),
            128 => Ok(IntegerBitSize::U128),
            _ => Err("Invalid bit size"),
        }
    }
}

impl std::fmt::Display for IntegerBitSize {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            IntegerBitSize::U0 => write!(f, "null"),
            IntegerBitSize::U1 => write!(f, "bool"),
            IntegerBitSize::U8 => write!(f, "u8"),
            IntegerBitSize::U16 => write!(f, "u16"),
            IntegerBitSize::U32 => write!(f, "u32"),
            IntegerBitSize::U64 => write!(f, "u64"),
            IntegerBitSize::U128 => write!(f, "u128"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, PartialOrd, Ord)]
pub enum BitSize {
    Field,
    Integer(IntegerBitSize),
}

impl BitSize {
    pub fn to_u32<F: AcirField>(self) -> u32 {
        match self {
            BitSize::Field => F::max_num_bits(),
            BitSize::Integer(bit_size) => bit_size.into(),
        }
    }

    pub fn try_from_u32<F: AcirField>(value: u32) -> Result<Self, &'static str> {
        if value == F::max_num_bits() {
            Ok(BitSize::Field)
        } else {
            Ok(BitSize::Integer(IntegerBitSize::try_from(value)?))
        }
    }
}

/// Lays out various ways an external foreign call's input and output data may be interpreted inside Brillig.
/// This data can either be an individual value or memory.
///
/// While we are usually agnostic to how memory is passed within Brillig,
/// this needs to be encoded somehow when dealing with an external system.
/// For simplicity, the extra type information is given right in the ForeignCall instructions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub enum ValueOrArray {
    /// A single value passed to or from an external call
    /// It is an 'immediate' value - used without dereferencing.
    /// For a foreign call input, the value is read directly from memory.
    /// For a foreign call output, the value is written directly to memory.
    MemoryAddress(MemoryAddress),
    /// An array passed to or from an external call
    /// In the case of a foreign call input, the array is read from this Brillig memory location + usize more cells.
    /// In the case of a foreign call output, the array is written to this Brillig memory location with the usize being here just as a sanity check for the size write.
    HeapArray(HeapArray),
    /// A vector passed to or from an external call
    /// In the case of a foreign call input, the vector is read from this Brillig memory location + as many cells as the 2nd address indicates.
    /// In the case of a foreign call output, the vector is written to this Brillig memory location and as 'size' cells, with size being stored in the second address.
    HeapVector(HeapVector),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrilligOpcode<F> {
    /// Takes the fields in addresses `lhs` and `rhs`
    /// Performs the specified binary operation
    /// and stores the value in the `result` address.  
    BinaryFieldOp {
        destination: MemoryAddress,
        op: BinaryFieldOp,
        lhs: MemoryAddress,
        rhs: MemoryAddress,
    },
    /// Takes the `bit_size` size integers in addresses `lhs` and `rhs`
    /// Performs the specified binary operation
    /// and stores the value in the `result` address.  
    BinaryIntOp {
        destination: MemoryAddress,
        op: BinaryIntOp,
        bit_size: IntegerBitSize,
        lhs: MemoryAddress,
        rhs: MemoryAddress,
    },
    Cast {
        destination: MemoryAddress,
        source: MemoryAddress,
        bit_size: BitSize,
    },
    JumpIfNot {
        condition: MemoryAddress,
        location: Label,
    },
    /// Sets the program counter to the value located at `destination`
    /// If the value at `condition` is non-zero
    JumpIf {
        condition: MemoryAddress,
        location: Label,
    },
    /// Sets the program counter to the label.
    Jump {
        location: Label,
    },
    /// Copies calldata after the offset to the specified address and length
    CalldataCopy {
        destination_address: MemoryAddress,
        size: usize,
        offset: usize,
    },
    /// We don't support dynamic jumps or calls
    /// See https://github.com/ethereum/aleth/issues/3404 for reasoning
    Call {
        location: Label,
    },
    Const {
        destination: MemoryAddress,
        bit_size: BitSize,
        value: F,
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
        /// Destination addresses (may be single values or memory pointers).
        destinations: Vec<ValueOrArray>,
        /// Destination value types
        destination_value_types: Vec<HeapValueType>,
        /// Input addresses (may be single values or memory pointers).
        inputs: Vec<ValueOrArray>,
        /// Input value types (for heap allocated structures indicates how to
        /// retrieve the elements)
        input_value_types: Vec<HeapValueType>,
    },
    Mov {
        destination: MemoryAddress,
        source: MemoryAddress,
    },
    /// destination = condition > 0 ? source_a : source_b
    ConditionalMov {
        destination: MemoryAddress,
        source_a: MemoryAddress,
        source_b: MemoryAddress,
        condition: MemoryAddress,
    },
    Load {
        destination: MemoryAddress,
        source_pointer: MemoryAddress,
    },
    Store {
        destination_pointer: MemoryAddress,
        source: MemoryAddress,
    },
    BlackBox(BlackBoxOp),
    /// Used to denote execution failure, returning data after the offset
    Trap {
        revert_data: HeapArray,
    },
    /// Stop execution, returning data after the offset
    Stop {
        return_data_offset: usize,
        return_data_size: usize,
    },
}

/// Binary fixed-length field expressions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryFieldOp {
    Add,
    Sub,
    Mul,
    /// Field division
    Div,
    /// Integer division
    IntegerDiv,
    /// (==) equal
    Equals,
    /// (<) Field less than
    LessThan,
    /// (<=) field less or equal
    LessThanEquals,
}

/// Binary fixed-length integer expressions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryIntOp {
    Add,
    Sub,
    Mul,
    Div,
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
