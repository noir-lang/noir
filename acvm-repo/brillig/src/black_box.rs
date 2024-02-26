use crate::{opcodes::HeapVector, HeapArray, MemoryAddress};
use serde::{Deserialize, Serialize};

/// These opcodes provide an equivalent of ACIR blackbox functions.
/// They are implemented as native functions in the VM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlackBoxOp {
    /// Calculates the SHA256 hash of the inputs.
    Sha256 {
        message: HeapVector,
        output: HeapArray,
    },
    /// Calculates the Blake2s hash of the inputs.
    Blake2s {
        message: HeapVector,
        output: HeapArray,
    },
    /// Calculates the Blake3 hash of the inputs.
    Blake3 {
        message: HeapVector,
        output: HeapArray,
    },
    /// Calculates the Keccak256 hash of the inputs.
    Keccak256 {
        message: HeapVector,
        output: HeapArray,
    },
    /// Keccak Permutation function of 1600 width
    Keccakf1600 {
        message: HeapVector,
        output: HeapArray,
    },
    /// Verifies a ECDSA signature over the secp256k1 curve.
    EcdsaSecp256k1 {
        hashed_msg: HeapVector,
        public_key_x: HeapArray,
        public_key_y: HeapArray,
        signature: HeapArray,
        result: MemoryAddress,
    },
    /// Verifies a ECDSA signature over the secp256r1 curve.
    EcdsaSecp256r1 {
        hashed_msg: HeapVector,
        public_key_x: HeapArray,
        public_key_y: HeapArray,
        signature: HeapArray,
        result: MemoryAddress,
    },
    /// Verifies a Schnorr signature over a curve which is "pairing friendly" with the curve on which the Brillig bytecode is defined.
    SchnorrVerify {
        public_key_x: MemoryAddress,
        public_key_y: MemoryAddress,
        message: HeapVector,
        signature: HeapVector,
        result: MemoryAddress,
    },
    /// Calculates a Pedersen commitment to the inputs.
    PedersenCommitment {
        inputs: HeapVector,
        domain_separator: MemoryAddress,
        output: HeapArray,
    },
    /// Calculates a Pedersen hash to the inputs.
    PedersenHash {
        inputs: HeapVector,
        domain_separator: MemoryAddress,
        output: MemoryAddress,
    },
    /// Performs scalar multiplication over the embedded curve.
    FixedBaseScalarMul {
        low: MemoryAddress,
        high: MemoryAddress,
        result: HeapArray,
    },
    /// Performs addition over the embedded curve.
    EmbeddedCurveAdd {
        input1_x: MemoryAddress,
        input1_y: MemoryAddress,
        input2_x: MemoryAddress,
        input2_y: MemoryAddress,
        result: HeapArray,
    },
    BigIntAdd {
        lhs: MemoryAddress,
        rhs: MemoryAddress,
        output: MemoryAddress,
    },
    BigIntSub {
        lhs: MemoryAddress,
        rhs: MemoryAddress,
        output: MemoryAddress,
    },
    BigIntMul {
        lhs: MemoryAddress,
        rhs: MemoryAddress,
        output: MemoryAddress,
    },
    BigIntDiv {
        lhs: MemoryAddress,
        rhs: MemoryAddress,
        output: MemoryAddress,
    },
    BigIntFromLeBytes {
        inputs: HeapVector,
        modulus: HeapVector,
        output: MemoryAddress,
    },
    BigIntToLeBytes {
        input: MemoryAddress,
        output: HeapVector,
    },
    Poseidon2Permutation {
        message: HeapVector,
        output: HeapArray,
        len: MemoryAddress,
    },
    Sha256Compression {
        input: HeapVector,
        hash_values: HeapVector,
        output: HeapArray,
    },
}
