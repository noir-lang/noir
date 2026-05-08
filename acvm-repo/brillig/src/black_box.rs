use crate::{HeapArray, MemoryAddress};
use msgpack_tagged::MsgpackTagged;
use serde::{Deserialize, Serialize};

/// These opcodes provide an equivalent of ACIR blackbox functions.
/// They are implemented as native functions in the VM.
/// For more information, see the ACIR blackbox functions in acir::circuit::opcodes::BlackBoxFuncCall
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize, MsgpackTagged)]
#[cfg_attr(feature = "arb", derive(proptest_derive::Arbitrary))]
pub enum BlackBoxOp {
    /// Encrypts a message using AES-128.
    #[tag(0)]
    AES128Encrypt {
        #[tag(0)]
        inputs: HeapArray,
        #[tag(1)]
        iv: HeapArray,
        #[tag(2)]
        key: HeapArray,
        #[tag(3)]
        outputs: HeapArray,
    },
    /// Calculates the Blake2s hash of the inputs.
    #[tag(1)]
    Blake2s {
        #[tag(0)]
        message: HeapArray,
        #[tag(1)]
        output: HeapArray,
    },
    /// Calculates the Blake3 hash of the inputs.
    #[tag(2)]
    Blake3 {
        #[tag(0)]
        message: HeapArray,
        #[tag(1)]
        output: HeapArray,
    },
    /// Keccak permutation function of 1600 width.
    #[tag(3)]
    Keccakf1600 {
        #[tag(0)]
        input: HeapArray,
        #[tag(1)]
        output: HeapArray,
    },
    /// Verifies an ECDSA signature over the secp256k1 curve.
    #[tag(4)]
    EcdsaSecp256k1 {
        #[tag(0)]
        hashed_msg: HeapArray,
        #[tag(1)]
        public_key_x: HeapArray,
        #[tag(2)]
        public_key_y: HeapArray,
        #[tag(3)]
        signature: HeapArray,
        #[tag(4)]
        result: MemoryAddress,
    },
    /// Verifies an ECDSA signature over the secp256r1 curve.
    #[tag(5)]
    EcdsaSecp256r1 {
        #[tag(0)]
        hashed_msg: HeapArray,
        #[tag(1)]
        public_key_x: HeapArray,
        #[tag(2)]
        public_key_y: HeapArray,
        #[tag(3)]
        signature: HeapArray,
        #[tag(4)]
        result: MemoryAddress,
    },
    /// Performs multi scalar multiplication over the embedded curve.
    #[tag(6)]
    MultiScalarMul {
        #[tag(0)]
        points: HeapArray,
        #[tag(1)]
        scalars: HeapArray,
        #[tag(2)]
        outputs: HeapArray,
    },
    /// Performs addition over the embedded curve.
    #[tag(7)]
    EmbeddedCurveAdd {
        #[tag(0)]
        input1_x: MemoryAddress,
        #[tag(1)]
        input1_y: MemoryAddress,
        #[tag(2)]
        input1_infinite: MemoryAddress,
        #[tag(3)]
        input2_x: MemoryAddress,
        #[tag(4)]
        input2_y: MemoryAddress,
        #[tag(5)]
        input2_infinite: MemoryAddress,
        #[tag(6)]
        result: HeapArray,
    },
    /// Applies the Poseidon2 permutation function to the given state,
    /// outputting the permuted state.
    #[tag(8)]
    Poseidon2Permutation {
        #[tag(0)]
        message: HeapArray,
        #[tag(1)]
        output: HeapArray,
    },
    /// Applies the SHA-256 compression function to the input message
    #[tag(9)]
    Sha256Compression {
        #[tag(0)]
        input: HeapArray,
        #[tag(1)]
        hash_values: HeapArray,
        #[tag(2)]
        output: HeapArray,
    },
    /// Returns a decomposition in `num_limbs` limbs of the given input over the given radix.
    ///
    /// - The value stored in `radix` must be in the range [2, 256]
    /// - `num_limbs` must be at least one if the value stored in `input` is not zero.
    /// - The value stored in `output_bits` must have a `bit_size` of one.
    ///   That value specifies whether we should decompose into bits. The value stored in
    ///   the `radix` address must be two if the value stored in `output_bits` is equal to one.
    ///
    /// Native to the Brillig VM and not supported as an ACIR black box function.
    #[tag(10)]
    ToRadix {
        #[tag(0)]
        input: MemoryAddress,
        #[tag(1)]
        radix: MemoryAddress,
        #[tag(2)]
        output_pointer: MemoryAddress,
        #[tag(3)]
        num_limbs: MemoryAddress,
        #[tag(4)]
        output_bits: MemoryAddress,
    },
}

impl std::fmt::Display for BlackBoxOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlackBoxOp::AES128Encrypt { inputs, iv, key, outputs } => {
                write!(
                    f,
                    "aes_128_encrypt(inputs: {inputs}, iv: {iv}, key: {key}, outputs: {outputs})"
                )
            }
            BlackBoxOp::Blake2s { message, output } => {
                write!(f, "blake2s(message: {message}, output: {output})")
            }
            BlackBoxOp::Blake3 { message, output } => {
                write!(f, "blake3(message: {message}, output: {output})")
            }
            BlackBoxOp::Keccakf1600 { input, output } => {
                write!(f, "keccakf1600(input: {input}, output: {output})")
            }
            BlackBoxOp::EcdsaSecp256k1 {
                hashed_msg,
                public_key_x,
                public_key_y,
                signature,
                result,
            } => {
                write!(
                    f,
                    "ecdsa_secp256k1(hashed_msg: {hashed_msg}, public_key_x: {public_key_x}, public_key_y: {public_key_y}, signature: {signature}, result: {result})"
                )
            }
            BlackBoxOp::EcdsaSecp256r1 {
                hashed_msg,
                public_key_x,
                public_key_y,
                signature,
                result,
            } => {
                write!(
                    f,
                    "ecdsa_secp256r1(hashed_msg: {hashed_msg}, public_key_x: {public_key_x}, public_key_y: {public_key_y}, signature: {signature}, result: {result})"
                )
            }
            BlackBoxOp::MultiScalarMul { points, scalars, outputs } => {
                write!(
                    f,
                    "multi_scalar_mul(points: {points}, scalars: {scalars}, outputs: {outputs})"
                )
            }
            BlackBoxOp::EmbeddedCurveAdd {
                input1_x,
                input1_y,
                input1_infinite,
                input2_x,
                input2_y,
                input2_infinite,
                result,
            } => {
                write!(
                    f,
                    "embedded_curve_add(input1_x: {input1_x}, input1_y: {input1_y}, input1_infinite: {input1_infinite}, input2_x: {input2_x}, input2_y: {input2_y}, input2_infinite: {input2_infinite}, result: {result})"
                )
            }
            BlackBoxOp::Poseidon2Permutation { message, output } => {
                write!(f, "poseidon2_permutation(message: {message}, output: {output})")
            }
            BlackBoxOp::Sha256Compression { input, hash_values, output } => {
                write!(
                    f,
                    "sha256_compression(input: {input}, hash_values: {hash_values}, output: {output})"
                )
            }
            BlackBoxOp::ToRadix { input, radix, output_pointer, num_limbs, output_bits } => {
                write!(
                    f,
                    "to_radix(input: {input}, radix: {radix}, num_limbs: {num_limbs}, output_pointer: {output_pointer}, output_bits: {output_bits})"
                )
            }
        }
    }
}
