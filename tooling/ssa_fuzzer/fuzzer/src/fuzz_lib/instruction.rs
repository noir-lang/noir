use libfuzzer_sys::arbitrary;
use libfuzzer_sys::arbitrary::Arbitrary;
use noir_ssa_fuzzer::typed_value::ValueType;
use serde::{Deserialize, Serialize};

#[derive(Arbitrary, Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct Array {
    pub(crate) size: usize,
    pub(crate) element_type: ValueType,
}

#[derive(Arbitrary, Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct Argument {
    /// Index of the argument in the context of stored variables of this type
    /// e.g. if we have variables with ids [0, 1] in u64 vector and variables with ids [5, 8] in fields vector
    /// Argument(Index(0), ValueType::U64) -> id 0
    /// Argument(Index(0), ValueType::Field) -> id 5
    /// Argument(Index(1), ValueType::Field) -> id 8
    pub(crate) index: usize,
    /// Type of the argument
    pub(crate) value_type: ValueType,
}

/// Represents set of instructions
///
/// For operations that take two arguments we ignore type of the second argument.
#[derive(Arbitrary, Debug, Clone, Serialize, Deserialize)]
pub(crate) enum Instruction {
    /// Addition of two values
    AddChecked { lhs: Argument, rhs: Argument },
    /// Subtraction of two values
    SubChecked { lhs: Argument, rhs: Argument },
    /// Multiplication of two values
    MulChecked { lhs: Argument, rhs: Argument },
    /// Division of two values
    Div { lhs: Argument, rhs: Argument },
    /// Equality comparison
    Eq { lhs: Argument, rhs: Argument },
    /// Modulo operation
    Mod { lhs: Argument, rhs: Argument },
    /// Bitwise NOT
    Not { lhs: Argument },
    /// Left shift
    Shl { lhs: Argument, rhs: Argument },
    /// Right shift
    Shr { lhs: Argument, rhs: Argument },
    /// Cast into type
    Cast { lhs: Argument, type_: ValueType },
    /// Bitwise AND
    And { lhs: Argument, rhs: Argument },
    /// Bitwise OR
    Or { lhs: Argument, rhs: Argument },
    /// Bitwise XOR
    Xor { lhs: Argument, rhs: Argument },

    /// Less than comparison
    Lt { lhs: Argument, rhs: Argument },

    /// constrain(lhs == lhs + rhs - rhs), doesn't insert constraint if idempotent_morphing_enabled=false
    /// uses only fields variables
    AddSubConstrain { lhs: usize, rhs: usize },
    /// constrain(lhs == lhs * rhs / rhs), doesn't insert constraint if idempotent_morphing_enabled=false
    /// uses only fields variables
    MulDivConstrain { lhs: usize, rhs: usize },

    /// Store value to mutable memory
    /// Allocates memory for Argument.value_type type with insert_allocate
    /// Stores value to memory with insert_store
    AddToMemory { lhs: Argument },
    /// Load value from mutable memory
    /// Loads value from memory with insert_load, choosing memory address for type Argument.value_type and index Argument.index
    /// Returns value of type Argument.value_type
    LoadFromMemory { memory_addr: Argument },
    /// Store value to mutable memory
    /// Stores value to memory with insert_store
    SetToMemory { memory_addr_index: usize, value: Argument },

    /// Create array, only type of first argument is used
    /// Other elements will be taken from stored variables of the same type
    CreateArray { elements_indices: Vec<usize>, element_type: ValueType, is_references: bool },
    /// Get element from array, index will be casted to u32, only for arrays without references
    ArrayGet { array_index: usize, index: Argument },
    /// Set element in array, index will be casted to u32, only for arrays without references
    /// Value will be casted to the type of the array
    ArraySet { array_index: usize, index: Argument, value_index: usize, mutable: bool },
    /// Get element from array, index is constant
    ArrayGetWithConstantIndex { array_index: usize, index: usize },
    /// Set element in array, index is constant
    /// Value will be casted to the type of the array
    ArraySetWithConstantIndex {
        array_index: usize,
        index: usize,
        value_index: usize,
        mutable: bool,
    },
}

/// Represents set of instructions
/// NOT EQUAL TO SSA BLOCK
#[derive(Arbitrary, Debug, Clone, Serialize, Deserialize)]
pub(crate) struct InstructionBlock {
    pub(crate) instructions: Vec<Instruction>,
}

#[derive(Clone)]
pub(crate) struct FunctionSignature {
    pub(crate) input_types: Vec<ValueType>,
    pub(crate) return_type: ValueType,
}
