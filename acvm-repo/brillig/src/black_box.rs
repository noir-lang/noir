use crate::{opcodes::HeapVector, HeapArray, RegisterIndex};
use serde::{Deserialize, Serialize};

/// These opcodes provide an equivalent of ACIR blackbox functions.
/// They are implemented as native functions in the VM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlackBoxOp {
    /// Calculates the SHA256 hash of the inputs.
    Sha256 { message: HeapVector, output: HeapArray },
    /// Calculates the Blake2s hash of the inputs.
    Blake2s { message: HeapVector, output: HeapArray },
    /// Calculates the Keccak256 hash of the inputs.
    Keccak256 { message: HeapVector, output: HeapArray },
    /// Hashes a set of inputs and applies the field modulus to the result
    /// to return a value which can be represented as a [`FieldElement`][acir_field::FieldElement]
    ///
    /// This is implemented using the `Blake2s` hash function.
    /// The "128" in the name specifies that this function should have 128 bits of security.
    HashToField128Security { message: HeapVector, output: RegisterIndex },
    /// Verifies a ECDSA signature over the secp256k1 curve.
    EcdsaSecp256k1 {
        hashed_msg: HeapVector,
        public_key_x: HeapArray,
        public_key_y: HeapArray,
        signature: HeapArray,
        result: RegisterIndex,
    },
    /// Verifies a ECDSA signature over the secp256r1 curve.
    EcdsaSecp256r1 {
        hashed_msg: HeapVector,
        public_key_x: HeapArray,
        public_key_y: HeapArray,
        signature: HeapArray,
        result: RegisterIndex,
    },
    /// Verifies a Schnorr signature over a curve which is "pairing friendly" with the curve on which the Brillig bytecode is defined.
    SchnorrVerify {
        public_key_x: RegisterIndex,
        public_key_y: RegisterIndex,
        message: HeapVector,
        signature: HeapVector,
        result: RegisterIndex,
    },
    /// Calculates a Pedersen commitment to the inputs.
    Pedersen { inputs: HeapVector, domain_separator: RegisterIndex, output: HeapArray },
    /// Performs scalar multiplication over the embedded curve.
    FixedBaseScalarMul { low: RegisterIndex, high: RegisterIndex, result: HeapArray },
}
