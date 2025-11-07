use libfuzzer_sys::arbitrary;
use libfuzzer_sys::arbitrary::Arbitrary;
use noir_ssa_fuzzer::typed_value::{NumericType, Type};
use serde::{Deserialize, Serialize};
use strum_macros::EnumCount;

#[derive(Arbitrary, Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct NumericArgument {
    /// Index of the argument in the context of stored variables of this type
    /// e.g. if we have variables with ids [0, 1] in u64 vector and variables with ids [5, 8] in fields vector
    /// Argument(Index(0), ValueType::U64) -> id 0
    /// Argument(Index(0), ValueType::Field) -> id 5
    /// Argument(Index(1), ValueType::Field) -> id 8
    pub(crate) index: usize,
    /// Type of the argument
    pub(crate) numeric_type: NumericType,
}

#[derive(Arbitrary, Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Argument {
    pub(crate) index: usize,
    pub(crate) value_type: Type,
}

#[derive(Arbitrary, Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub(crate) struct Scalar {
    pub(crate) field_lo_idx: usize,
    pub(crate) field_hi_idx: usize,
}

#[derive(Arbitrary, Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub(crate) struct Point {
    pub(crate) scalar: Scalar,
    /// If true, the point will be derived from scalar multiplication using [`noir_ssa_fuzzer::builder::FuzzerBuilder::base_scalar_mul`]
    /// Otherwise, the point will be derived from scalar values using [`noir_ssa_fuzzer::builder::FuzzerBuilder::create_point_from_scalar`]
    pub(crate) derive_from_scalar_mul: bool,
    pub(crate) is_infinite: bool,
}

pub(crate) type PointAndScalar = (Point, Scalar);

/// Represents set of instructions
///
/// For operations that take two arguments we ignore type of the second argument.
#[derive(Arbitrary, Debug, Clone, Serialize, Deserialize, EnumCount)]
pub(crate) enum Instruction {
    /// Addition of two values
    AddChecked { lhs: NumericArgument, rhs: NumericArgument },
    /// Subtraction of two values
    SubChecked { lhs: NumericArgument, rhs: NumericArgument },
    /// Multiplication of two values
    MulChecked { lhs: NumericArgument, rhs: NumericArgument },
    /// Division of two values
    Div { lhs: NumericArgument, rhs: NumericArgument },
    /// Equality comparison
    Eq { lhs: NumericArgument, rhs: NumericArgument },
    /// Modulo operation
    Mod { lhs: NumericArgument, rhs: NumericArgument },
    /// Bitwise NOT
    Not { lhs: NumericArgument },
    /// Left shift
    Shl { lhs: NumericArgument, rhs: NumericArgument },
    /// Right shift
    Shr { lhs: NumericArgument, rhs: NumericArgument },
    /// Cast into type
    Cast { lhs: NumericArgument, type_: NumericType },
    /// Bitwise AND
    And { lhs: NumericArgument, rhs: NumericArgument },
    /// Bitwise OR
    Or { lhs: NumericArgument, rhs: NumericArgument },
    /// Bitwise XOR
    Xor { lhs: NumericArgument, rhs: NumericArgument },

    /// Less than comparison
    Lt { lhs: NumericArgument, rhs: NumericArgument },

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
    /// If reference to this type is not found, allocates memory for it and stores value to it
    SetToMemory { memory_addr_index: usize, value: Argument },

    /// Create array, only type of first argument is used
    /// Other elements will be taken from stored variables of the same type
    CreateArray { elements_indices: Vec<usize>, element_type: Type },
    /// Get element from array, index will be casted to u32, only for arrays without references
    /// If safe_index is true, index will be taken modulo the size of the array
    ArrayGet { array_index: usize, index: NumericArgument, safe_index: bool },
    /// Set element in array, index will be casted to u32, only for arrays without references
    /// Value will be cast to the type of the array
    /// If safe_index is true, index will be taken modulo the size of the array
    ArraySet { array_index: usize, index: NumericArgument, value_index: usize, safe_index: bool },
    /// Get element from array, index is constant
    /// If safe_index is true, index will be taken modulo the size of the array
    ArrayGetWithConstantIndex { array_index: usize, index: usize, safe_index: bool },
    /// Set element in array, index is constant
    /// Value will be cast to the type of the array
    /// If safe_index is true, index will be taken modulo the size of the array
    ArraySetWithConstantIndex {
        array_index: usize,
        index: usize,
        value_index: usize,
        safe_index: bool,
    },

    /// Field to bytes to field
    /// Takes field, converts it to le_bytes
    /// Then converts the le_bytes to field and stores it in the context
    FieldToBytesToField { field_idx: usize },

    /// Blake2s hash
    /// Takes field, converts it to le_bytes of the size specified by `limbs_count`
    /// Then hashes it with blake2s and stores the hash from le_bytes in the context
    Blake2sHash { field_idx: usize, limbs_count: u8 },

    /// Blake3 hash
    /// Takes field, converts it to le_bytes of the size specified by `limbs_count`
    /// Then hashes it with blake3 and stores the hash from le_bytes in the context
    Blake3Hash { field_idx: usize, limbs_count: u8 },

    /// Keccakf1600 hash
    /// Takes array of u64 values and permutes it with keccakf1600
    /// Stores the permuted array in the context
    /// If `load_elements_of_array` is true, loads all elements of the permuted array into defined variables
    Keccakf1600Hash { u64_indices: [usize; 25], load_elements_of_array: bool },

    /// AES-128 encrypt
    /// Takes input key and iv as fields, converts them to u8 arrays
    /// Input is converted to u8 array of size `input_limbs_count`
    /// Encrypts the input with AES-128 and converts encrypted array to field and stores it in the context
    Aes128Encrypt { input_idx: usize, input_limbs_count: u8, key_idx: usize, iv_idx: usize },

    /// SHA-256 compression
    /// Takes input and state as arrays of u32 values
    /// Compresses the input with SHA-256 and stores the result in the context
    /// If `load_elements_of_array` is true, loads all elements of the compressed array into defined variables
    Sha256Compression {
        input_indices: [usize; 16],
        state_indices: [usize; 8],
        load_elements_of_array: bool,
    },

    /// Point addition
    PointAdd { p1: Point, p2: Point, predicate: bool },

    /// Multi-scalar multiplication
    MultiScalarMul { points_and_scalars: Vec<PointAndScalar>, predicate: bool },

    /// ECDSA secp256r1
    EcdsaSecp256r1 {
        msg: Vec<u8>,
        corrupt_hash: bool,
        corrupt_pubkey_x: bool,
        corrupt_pubkey_y: bool,
        corrupt_signature: bool,
        predicate: bool,
    },

    /// ECDSA secp256k1
    EcdsaSecp256k1 {
        msg: Vec<u8>,
        corrupt_hash: bool,
        corrupt_pubkey_x: bool,
        corrupt_pubkey_y: bool,
        corrupt_signature: bool,
        predicate: bool,
    },
}

/// Default instruction is XOR of two boolean values
///
/// Only used for mutations
impl Default for Instruction {
    fn default() -> Self {
        Self::Xor {
            lhs: NumericArgument { index: 0, numeric_type: NumericType::Boolean },
            rhs: NumericArgument { index: 0, numeric_type: NumericType::Boolean },
        }
    }
}

/// Represents set of instructions
/// NOT EQUAL TO SSA BLOCK
#[derive(Arbitrary, Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct InstructionBlock {
    pub(crate) instructions: Vec<Instruction>,
}

#[derive(Clone)]
pub(crate) struct FunctionInfo {
    pub(crate) input_types: Vec<Type>,
    pub(crate) return_type: Type,
    /// Max size of unrolled loops in the function
    pub(crate) max_unrolled_size: usize,
}
