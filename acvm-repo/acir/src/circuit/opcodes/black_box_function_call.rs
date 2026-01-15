//! Black box functions are ACIR opcodes which rely on backends implementing
//! support for specialized constraints.
//! This makes certain zk-snark unfriendly computations cheaper than if they were
//! implemented in more basic constraints.

use std::collections::BTreeSet;

use crate::BlackBoxFunc;
use crate::native_types::Witness;

use acir_field::AcirField;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

/// Enumeration for black box function inputs
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[cfg_attr(feature = "arb", derive(proptest_derive::Arbitrary))]
pub enum FunctionInput<F> {
    /// A constant field element
    Constant(F),
    /// A witness element, representing dynamic inputs
    Witness(Witness),
}

impl<F: std::fmt::Display> std::fmt::Display for FunctionInput<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            FunctionInput::Constant(constant) => {
                write!(f, "{constant}")
            }
            FunctionInput::Witness(witness) => {
                write!(f, "w{}", witness.0)
            }
        }
    }
}

impl<F> FunctionInput<F> {
    pub fn is_constant(&self) -> bool {
        match self {
            FunctionInput::Constant(_) => true,
            FunctionInput::Witness(_) => false,
        }
    }

    pub fn to_witness(&self) -> Witness {
        match self {
            FunctionInput::Constant(_) => unreachable!("ICE - Expected Witness"),
            FunctionInput::Witness(witness) => *witness,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Error)]
#[error("FunctionInput value has too many bits: value: {value}, {value_num_bits} >= {max_bits}")]
pub struct InvalidInputBitSize {
    pub value: String,
    pub value_num_bits: u32,
    pub max_bits: u32,
}

/// These opcodes represent a specialized computation.
/// Even if any computation can be done using only assert-zero opcodes,
/// it is not always efficient.
/// Some proving systems, can implement several computations more efficiently using
/// techniques such as custom gates and lookup tables.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum BlackBoxFuncCall<F> {
    /// Ciphers (encrypts) the provided plaintext using AES128 in CBC mode,
    /// padding the input using PKCS#7.
    /// - inputs: byte array `[u8; N]`
    /// - iv: initialization vector `[u8; 16]`
    /// - key: user key `[u8; 16]`
    /// - outputs: byte vector `[u8]` of length `input.len() + (16 - input.len() % 16)`
    AES128Encrypt {
        inputs: Vec<FunctionInput<F>>,
        iv: Box<[FunctionInput<F>; 16]>,
        key: Box<[FunctionInput<F>; 16]>,
        outputs: Vec<Witness>,
    },
    /// Performs the bitwise AND of `lhs` and `rhs`. `bit_size` must be the same for
    /// both inputs.
    /// - lhs: (witness, bit_size)
    /// - rhs: (witness, bit_size)
    /// - output: a witness whose value is constrained to be lhs AND rhs, as
    ///   bit_size bit integers
    AND { lhs: FunctionInput<F>, rhs: FunctionInput<F>, num_bits: u32, output: Witness },
    /// Performs the bitwise XOR of `lhs` and `rhs`. `bit_size` must be the same for
    /// both inputs.
    /// - lhs: (witness, bit_size)
    /// - rhs: (witness, bit_size)
    /// - output: a witness whose value is constrained to be lhs XOR rhs, as
    ///   bit_size bit integers
    XOR { lhs: FunctionInput<F>, rhs: FunctionInput<F>, num_bits: u32, output: Witness },
    /// Range constraint to ensure that a witness
    /// can be represented in the specified number of bits.
    /// - input: (witness, bit_size)
    RANGE { input: FunctionInput<F>, num_bits: u32 },
    /// Computes the Blake2s hash of the inputs, as specified in
    /// <https://tools.ietf.org/html/rfc7693>
    /// - inputs are a byte array, i.e a vector of (witness, 8)
    /// - output is a byte array of length 32, i.e. an array of 32
    ///   (witness, 8), constrained to be the blake2s of the inputs.
    Blake2s { inputs: Vec<FunctionInput<F>>, outputs: Box<[Witness; 32]> },
    /// Computes the Blake3 hash of the inputs
    /// - inputs are a byte array, i.e a vector of (witness, 8)
    /// - output is a byte array of length 32, i.e an array of 32
    ///   (witness, 8), constrained to be the blake3 of the inputs.
    Blake3 { inputs: Vec<FunctionInput<F>>, outputs: Box<[Witness; 32]> },
    /// Verifies a ECDSA signature over the secp256k1 curve.
    /// - inputs:
    ///     - x coordinate of public key as 32 bytes
    ///     - y coordinate of public key as 32 bytes
    ///     - the signature, as a 64 bytes array
    ///       The signature internally will be represented as `(r, s)`,
    ///       where `r` and `s` are fixed-sized big endian scalar values.
    ///       As the `secp256k1` has a 256-bit modulus, we have a 64 byte signature
    ///       while `r` and `s` will both be 32 bytes.
    ///       We expect `s` to be normalized. This means given the curve's order,
    ///       `s` should be less than or equal to `order / 2`.
    ///       This is done to prevent malleability.
    ///       For more context regarding malleability you can reference BIP 0062.
    ///     - the hash of the message, as a vector of bytes
    /// - output: 0 for failure and 1 for success
    ///
    /// Expected backend behavior:
    /// - The backend MAY fail to prove this opcode if the public key is not on the secp256k1 curve.
    ///    - Otherwise the backend MUST constrain the output to be false.
    /// - The backend MUST constrain the output to be false if `s` is not normalized.
    /// - The backend MUST constrain the output to match the signature's validity.
    EcdsaSecp256k1 {
        public_key_x: Box<[FunctionInput<F>; 32]>,
        public_key_y: Box<[FunctionInput<F>; 32]>,
        #[serde(
            serialize_with = "serialize_big_array",
            deserialize_with = "deserialize_big_array_into_box"
        )]
        signature: Box<[FunctionInput<F>; 64]>,
        hashed_message: Box<[FunctionInput<F>; 32]>,
        predicate: FunctionInput<F>,
        output: Witness,
    },
    /// Verifies a ECDSA signature over the secp256r1 curve.
    ///
    /// Same as EcdsaSecp256k1, but done over another curve.
    EcdsaSecp256r1 {
        public_key_x: Box<[FunctionInput<F>; 32]>,
        public_key_y: Box<[FunctionInput<F>; 32]>,
        #[serde(
            serialize_with = "serialize_big_array",
            deserialize_with = "deserialize_big_array_into_box"
        )]
        signature: Box<[FunctionInput<F>; 64]>,
        hashed_message: Box<[FunctionInput<F>; 32]>,
        predicate: FunctionInput<F>,
        output: Witness,
    },
    /// Multiple scalar multiplication (MSM) with a variable base/input point
    /// (P) of the embedded curve. An MSM multiplies the points and scalars and
    /// sums the results.
    /// - input:
    ///     - points (witness, N) a vector of x and y coordinates of input
    ///     - points `[x1, y1, x2, y2,...]`.
    ///     - scalars (witness, N) a vector of low and high limbs of input
    ///     - scalars `[s1_low, s1_high, s2_low, s2_high, ...]`. (witness, N)
    ///       For Barretenberg, they must both be less than 128 bits.
    /// - output:
    ///     - a tuple of `x` and `y` coordinates of output
    ///       points computed as `s_low*P+s_high*2^{128}*P`
    ///
    /// Because the Grumpkin scalar field is bigger than the ACIR field, we
    /// provide 2 ACIR fields representing the low and high parts of the Grumpkin
    /// scalar $a$: `a=low+high*2^{128}`, with `low, high < 2^{128}`
    MultiScalarMul {
        points: Vec<FunctionInput<F>>,
        scalars: Vec<FunctionInput<F>>,
        predicate: FunctionInput<F>,
        outputs: (Witness, Witness, Witness),
    },
    /// Addition over the embedded curve on which the witness is defined
    /// The opcode makes the following assumptions but does not enforce them because
    /// it is more efficient to do it only when required. For instance, adding two
    /// points that are on the curve it guarantee to give a point on the curve.
    ///
    /// It assumes that the points are on the curve.
    /// If the inputs are the same witnesses index, it will perform a doubling,
    /// If not, it assumes that the points' x-coordinates are not equal.
    /// It also assumes neither point is the infinity point.
    EmbeddedCurveAdd {
        input1: Box<[FunctionInput<F>; 3]>,
        input2: Box<[FunctionInput<F>; 3]>,
        predicate: FunctionInput<F>,
        outputs: (Witness, Witness, Witness),
    },
    /// Keccak Permutation function of width 1600
    /// - inputs: An array of 25 64-bit Keccak lanes that represent a keccak sponge of 1600 bits
    /// - outputs: The result of a keccak f1600 permutation on the input state. Also an array of 25 Keccak lanes.
    Keccakf1600 { inputs: Box<[FunctionInput<F>; 25]>, outputs: Box<[Witness; 25]> },
    /// Computes a recursive aggregation object when verifying a proof inside
    /// another circuit.
    /// The outputted aggregation object will then be either checked in a
    /// top-level verifier or aggregated upon again.
    /// The aggregation object should be maintained by the backend implementer.
    ///
    /// This opcode prepares the verification of the final proof.
    /// In order to fully verify a recursive proof, some operations may still be required
    /// to be done by the final verifier (e.g. a pairing check).
    /// This is why this black box function does not say if verification is passing or not.
    /// It delays the expensive part of verification out of the SNARK
    /// and leaves it to the final verifier outside of the SNARK circuit.
    ///
    /// This opcode also verifies that the key_hash is indeed a hash of verification_key,
    /// allowing the user to use the verification key as private inputs and only
    /// have the key_hash as public input, which is more performant.
    ///
    /// **Warning: the key hash logic does not need to be part of the black box and subject to be removed.**
    ///
    /// If one of the recursive proofs you verify with the black box function fails to
    /// verify, then the verification of the final proof of the main ACIR program will
    /// ultimately fail.
    RecursiveAggregation {
        /// Verification key of the circuit being verified
        verification_key: Vec<FunctionInput<F>>,
        proof: Vec<FunctionInput<F>>,
        /// These represent the public inputs of the proof we are verifying
        /// They should be checked against in the circuit after construction
        /// of a new aggregation state
        public_inputs: Vec<FunctionInput<F>>,
        /// A key hash is used to check the validity of the verification key.
        /// The circuit implementing this opcode can use this hash to ensure that the
        /// key provided to the circuit matches the key produced by the circuit creator
        key_hash: FunctionInput<F>,
        /// Backend-specific proof type constant.
        /// The proof field is agnostic and can come from witness inputs.
        /// However, a backend may have many different verifiers which affect
        /// the circuit construction.
        /// In order for a backend to construct the correct recursive verifier
        /// it expects the user to specify a proof type.
        proof_type: u32,
        /// A predicate (true or false) to disable the recursive verification
        predicate: FunctionInput<F>,
        /// Output witnesses for the return_data commitment from the verified proof.
        /// This is a G1 point represented as 4 field elements (x_lo, x_hi, y_lo, y_hi).
        /// This commitment binds to the data that flows into the verifying circuit's
        /// corresponding call_data column via the databus consistency check.
        output: Vec<Witness>,
    },
    /// Applies the Poseidon2 permutation function to the given state,
    /// outputting the permuted state.
    Poseidon2Permutation {
        /// Input state for the permutation of Poseidon2
        inputs: Vec<FunctionInput<F>>,
        /// Permuted state
        outputs: Vec<Witness>,
    },
    /// Applies the SHA-256 compression function to the input message
    ///
    /// # Arguments
    ///
    /// * `inputs` - input message block
    /// * `hash_values` - state from the previous compression
    /// * `outputs` - result of the input compressed into 256 bits
    Sha256Compression {
        /// 512 bits of the input message, represented by 16 u32s
        inputs: Box<[FunctionInput<F>; 16]>,
        /// Vector of 8 u32s used to compress the input
        hash_values: Box<[FunctionInput<F>; 8]>,
        /// Output of the compression, represented by 8 u32s
        outputs: Box<[Witness; 8]>,
    },
}

impl<F> BlackBoxFuncCall<F> {
    pub fn get_black_box_func(&self) -> BlackBoxFunc {
        match self {
            BlackBoxFuncCall::AES128Encrypt { .. } => BlackBoxFunc::AES128Encrypt,
            BlackBoxFuncCall::AND { .. } => BlackBoxFunc::AND,
            BlackBoxFuncCall::XOR { .. } => BlackBoxFunc::XOR,
            BlackBoxFuncCall::RANGE { .. } => BlackBoxFunc::RANGE,
            BlackBoxFuncCall::Blake2s { .. } => BlackBoxFunc::Blake2s,
            BlackBoxFuncCall::Blake3 { .. } => BlackBoxFunc::Blake3,
            BlackBoxFuncCall::EcdsaSecp256k1 { .. } => BlackBoxFunc::EcdsaSecp256k1,
            BlackBoxFuncCall::EcdsaSecp256r1 { .. } => BlackBoxFunc::EcdsaSecp256r1,
            BlackBoxFuncCall::MultiScalarMul { .. } => BlackBoxFunc::MultiScalarMul,
            BlackBoxFuncCall::EmbeddedCurveAdd { .. } => BlackBoxFunc::EmbeddedCurveAdd,
            BlackBoxFuncCall::Keccakf1600 { .. } => BlackBoxFunc::Keccakf1600,
            BlackBoxFuncCall::RecursiveAggregation { .. } => BlackBoxFunc::RecursiveAggregation,
            BlackBoxFuncCall::Poseidon2Permutation { .. } => BlackBoxFunc::Poseidon2Permutation,
            BlackBoxFuncCall::Sha256Compression { .. } => BlackBoxFunc::Sha256Compression,
        }
    }

    pub fn name(&self) -> &str {
        self.get_black_box_func().name()
    }

    pub fn bit_size(&self) -> Option<u32> {
        match self {
            BlackBoxFuncCall::AND { num_bits, .. }
            | BlackBoxFuncCall::XOR { num_bits, .. }
            | BlackBoxFuncCall::RANGE { num_bits, .. } => Some(*num_bits),
            _ => None,
        }
    }

    pub fn get_outputs_vec(&self) -> Vec<Witness> {
        match self {
            BlackBoxFuncCall::Blake2s { outputs, .. }
            | BlackBoxFuncCall::Blake3 { outputs, .. } => outputs.to_vec(),

            BlackBoxFuncCall::Keccakf1600 { outputs, .. } => outputs.to_vec(),

            BlackBoxFuncCall::Sha256Compression { outputs, .. } => outputs.to_vec(),

            BlackBoxFuncCall::AES128Encrypt { outputs, .. }
            | BlackBoxFuncCall::Poseidon2Permutation { outputs, .. } => outputs.to_vec(),

            BlackBoxFuncCall::AND { output, .. }
            | BlackBoxFuncCall::XOR { output, .. }
            | BlackBoxFuncCall::EcdsaSecp256k1 { output, .. }
            | BlackBoxFuncCall::EcdsaSecp256r1 { output, .. } => vec![*output],
            BlackBoxFuncCall::MultiScalarMul { outputs, .. }
            | BlackBoxFuncCall::EmbeddedCurveAdd { outputs, .. } => {
                vec![outputs.0, outputs.1, outputs.2]
            }
            BlackBoxFuncCall::RANGE { .. } => {
                vec![]
            }
            BlackBoxFuncCall::RecursiveAggregation { output, .. } => output.to_vec(),
        }
    }
}

impl<F: Copy + AcirField> BlackBoxFuncCall<F> {
    pub fn get_inputs_vec(&self) -> Vec<FunctionInput<F>> {
        match self {
            BlackBoxFuncCall::Blake2s { inputs, outputs: _ }
            | BlackBoxFuncCall::Blake3 { inputs, outputs: _ }
            | BlackBoxFuncCall::Poseidon2Permutation { inputs, outputs: _ } => inputs.to_vec(),

            BlackBoxFuncCall::Keccakf1600 { inputs, outputs: _ } => inputs.to_vec(),
            BlackBoxFuncCall::AES128Encrypt { inputs, iv, key, outputs: _ } => {
                [inputs, iv.as_slice(), key.as_slice()].concat()
            }
            BlackBoxFuncCall::Sha256Compression { inputs, hash_values, outputs: _ } => {
                [inputs.as_slice(), hash_values.as_slice()].concat()
            }
            BlackBoxFuncCall::AND { lhs, rhs, output: _, num_bits: _ }
            | BlackBoxFuncCall::XOR { lhs, rhs, output: _, num_bits: _ } => {
                vec![*lhs, *rhs]
            }
            BlackBoxFuncCall::RANGE { input, num_bits: _ } => vec![*input],

            BlackBoxFuncCall::MultiScalarMul { points, scalars, predicate, outputs: _ } => {
                [points.as_slice(), scalars.as_slice(), &[*predicate]].concat()
            }
            BlackBoxFuncCall::EmbeddedCurveAdd { input1, input2, predicate, outputs: _ } => {
                vec![input1[0], input1[1], input1[2], input2[0], input2[1], input2[2], *predicate]
            }
            BlackBoxFuncCall::EcdsaSecp256k1 {
                public_key_x,
                public_key_y,
                signature,
                hashed_message,
                predicate,
                output: _,
            } => [
                public_key_x.as_slice(),
                public_key_y.as_slice(),
                signature.as_slice(),
                hashed_message.as_slice(),
                &[*predicate],
            ]
            .concat(),
            BlackBoxFuncCall::EcdsaSecp256r1 {
                public_key_x,
                public_key_y,
                signature,
                hashed_message,
                predicate,
                output: _,
            } => [
                public_key_x.as_slice(),
                public_key_y.as_slice(),
                signature.as_slice(),
                hashed_message.as_slice(),
                &[*predicate],
            ]
            .concat(),
            BlackBoxFuncCall::RecursiveAggregation {
                verification_key: key,
                proof,
                public_inputs,
                key_hash,
                proof_type: _,
                predicate,
                output: _,
            } => [key.as_slice(), proof, public_inputs, &[*key_hash], &[*predicate]].concat(),
        }
    }

    pub fn get_input_witnesses(&self) -> BTreeSet<Witness> {
        let mut result = BTreeSet::new();
        for input in self.get_inputs_vec() {
            if let FunctionInput::Witness(w) = input {
                result.insert(w);
            }
        }
        result
    }

    pub fn get_predicate(&self) -> Option<Witness> {
        let predicate = match self {
            BlackBoxFuncCall::AES128Encrypt { .. }
            | BlackBoxFuncCall::AND { .. }
            | BlackBoxFuncCall::XOR { .. }
            | BlackBoxFuncCall::RANGE { .. }
            | BlackBoxFuncCall::Blake2s { .. }
            | BlackBoxFuncCall::Blake3 { .. }
            | BlackBoxFuncCall::Keccakf1600 { .. }
            | BlackBoxFuncCall::Poseidon2Permutation { .. }
            | BlackBoxFuncCall::Sha256Compression { .. } => FunctionInput::Constant(F::one()),
            BlackBoxFuncCall::EcdsaSecp256k1 { predicate, .. }
            | BlackBoxFuncCall::EcdsaSecp256r1 { predicate, .. }
            | BlackBoxFuncCall::MultiScalarMul { predicate, .. }
            | BlackBoxFuncCall::EmbeddedCurveAdd { predicate, .. }
            | BlackBoxFuncCall::RecursiveAggregation { predicate, .. } => *predicate,
        };
        if predicate.is_constant() { None } else { Some(predicate.to_witness()) }
    }
}

impl<F: std::fmt::Display + Copy> std::fmt::Display for BlackBoxFuncCall<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let uppercase_name = self.name().to_uppercase();
        write!(f, "BLACKBOX::{uppercase_name} ")?;

        match self {
            BlackBoxFuncCall::AES128Encrypt { inputs, iv, key, outputs } => {
                let inputs = slice_to_string(inputs);
                let iv = slice_to_string(&iv.to_vec());
                let key = slice_to_string(&key.to_vec());
                let outputs = slice_to_string(outputs);
                write!(f, "inputs: {inputs}, iv: {iv}, key: {key}, outputs: {outputs}")?;
            }
            BlackBoxFuncCall::AND { lhs, rhs, num_bits, output }
            | BlackBoxFuncCall::XOR { lhs, rhs, num_bits, output } => {
                write!(f, "lhs: {lhs}, rhs: {rhs}, output: {output}, bits: {num_bits}")?;
            }
            BlackBoxFuncCall::RANGE { input, num_bits } => {
                write!(f, "input: {input}, bits: {num_bits}")?;
            }
            BlackBoxFuncCall::Blake2s { inputs, outputs }
            | BlackBoxFuncCall::Blake3 { inputs, outputs } => {
                let inputs = slice_to_string(inputs);
                let outputs = slice_to_string(&outputs.to_vec());
                write!(f, "inputs: {inputs}, outputs: {outputs}")?;
            }
            BlackBoxFuncCall::EcdsaSecp256k1 {
                public_key_x,
                public_key_y,
                signature,
                hashed_message,
                predicate,
                output,
            }
            | BlackBoxFuncCall::EcdsaSecp256r1 {
                public_key_x,
                public_key_y,
                signature,
                hashed_message,
                predicate,
                output,
            } => {
                let public_key_x = slice_to_string(&public_key_x.to_vec());
                let public_key_y = slice_to_string(&public_key_y.to_vec());
                let signature = slice_to_string(&signature.to_vec());
                let hashed_message = slice_to_string(&hashed_message.to_vec());
                write!(
                    f,
                    "public_key_x: {public_key_x}, public_key_y: {public_key_y}, signature: {signature}, hashed_message: {hashed_message}, predicate: {predicate}, output: {output}"
                )?;
            }
            BlackBoxFuncCall::MultiScalarMul { points, scalars, predicate, outputs } => {
                let points = slice_to_string(points);
                let scalars = slice_to_string(scalars);
                write!(
                    f,
                    "points: {points}, scalars: {scalars}, predicate: {predicate}, outputs: [{}, {}, {}]",
                    outputs.0, outputs.1, outputs.2
                )?;
            }
            BlackBoxFuncCall::EmbeddedCurveAdd { input1, input2, predicate, outputs } => {
                let input1 = slice_to_string(&input1.to_vec());
                let input2 = slice_to_string(&input2.to_vec());
                write!(
                    f,
                    "input1: {input1}, input2: {input2}, predicate: {predicate}, outputs: [{}, {}, {}]",
                    outputs.0, outputs.1, outputs.2
                )?;
            }
            BlackBoxFuncCall::Keccakf1600 { inputs, outputs } => {
                let inputs = slice_to_string(&inputs.to_vec());
                let outputs = slice_to_string(&outputs.to_vec());
                write!(f, "inputs: {inputs}, outputs: {outputs}")?;
            }
            BlackBoxFuncCall::RecursiveAggregation {
                verification_key,
                proof,
                public_inputs,
                key_hash,
                proof_type,
                predicate,
                output,
            } => {
                let verification_key = slice_to_string(verification_key);
                let proof = slice_to_string(proof);
                let public_inputs = slice_to_string(public_inputs);
                let output = slice_to_string(output);
                write!(
                    f,
                    "verification_key: {verification_key}, proof: {proof}, public_inputs: {public_inputs}, key_hash: {key_hash}, proof_type: {proof_type}, predicate: {predicate}, output: {output}"
                )?;
            }
            BlackBoxFuncCall::Poseidon2Permutation { inputs, outputs } => {
                let inputs = slice_to_string(inputs);
                let outputs = slice_to_string(outputs);
                write!(f, "inputs: {inputs}, outputs: {outputs}")?;
            }
            BlackBoxFuncCall::Sha256Compression { inputs, hash_values, outputs } => {
                let inputs = slice_to_string(&inputs.to_vec());
                let hash_values = slice_to_string(&hash_values.to_vec());
                let outputs = slice_to_string(&outputs.to_vec());
                write!(f, "inputs: {inputs}, hash_values: {hash_values}, outputs: {outputs}")?;
            }
        }

        Ok(())
    }
}

fn slice_to_string<D: std::fmt::Display>(inputs: &[D]) -> String {
    let inputs = inputs.iter().map(|i| i.to_string()).collect::<Vec<String>>().join(", ");
    format!("[{inputs}]")
}

impl<F: std::fmt::Display + Copy> std::fmt::Debug for BlackBoxFuncCall<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

fn serialize_big_array<S, F: Serialize>(
    big_array: &[FunctionInput<F>; 64],
    s: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use serde_big_array::BigArray;

    (*big_array).serialize(s)
}

fn deserialize_big_array_into_box<'de, D, F: Deserialize<'de>>(
    deserializer: D,
) -> Result<Box<[FunctionInput<F>; 64]>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde_big_array::BigArray;

    let big_array: [FunctionInput<F>; 64] = BigArray::deserialize(deserializer)?;
    Ok(Box::new(big_array))
}

#[cfg(test)]
mod tests {

    use crate::{circuit::Opcode, native_types::Witness};
    use acir_field::{AcirField, FieldElement};

    use super::{BlackBoxFuncCall, FunctionInput};

    fn keccakf1600_opcode<F: AcirField>() -> Opcode<F> {
        let inputs: Box<[FunctionInput<F>; 25]> =
            Box::new(std::array::from_fn(|i| FunctionInput::Witness(Witness(i as u32 + 1))));
        let outputs: Box<[Witness; 25]> = Box::new(std::array::from_fn(|i| Witness(i as u32 + 26)));

        Opcode::BlackBoxFuncCall(BlackBoxFuncCall::Keccakf1600 { inputs, outputs })
    }

    #[test]
    fn keccakf1600_serialization_roundtrip() {
        use crate::serialization::{msgpack_deserialize, msgpack_serialize};

        let opcode = keccakf1600_opcode::<FieldElement>();
        let buf = msgpack_serialize(&opcode, true).unwrap();
        let recovered_opcode = msgpack_deserialize(&buf).unwrap();
        assert_eq!(opcode, recovered_opcode);
    }
}

#[cfg(feature = "arb")]
mod arb {
    use acir_field::AcirField;
    use proptest::prelude::*;

    use crate::native_types::Witness;

    use super::{BlackBoxFuncCall, FunctionInput};

    // Implementing this separately because trying to derive leads to stack overflow.
    impl<F> Arbitrary for BlackBoxFuncCall<F>
    where
        F: AcirField + Arbitrary,
    {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            let input = any::<FunctionInput<F>>();
            let input_vec = any::<Vec<FunctionInput<F>>>();
            let input_arr_3 = any::<Box<[FunctionInput<F>; 3]>>();
            let input_arr_8 = any::<Box<[FunctionInput<F>; 8]>>();
            let input_arr_16 = any::<Box<[FunctionInput<F>; 16]>>();
            let input_arr_25 = any::<Box<[FunctionInput<F>; 25]>>();
            let input_arr_32 = any::<Box<[FunctionInput<F>; 32]>>();
            let input_arr_64 = any::<Box<[FunctionInput<F>; 64]>>();
            let witness = any::<Witness>();
            let witness_vec = any::<Vec<Witness>>();
            let witness_arr_8 = any::<Box<[Witness; 8]>>();
            let witness_arr_25 = any::<Box<[Witness; 25]>>();
            let witness_arr_32 = any::<Box<[Witness; 32]>>();

            let case_aes128_encrypt = (
                input_vec.clone(),
                input_arr_16.clone(),
                input_arr_16.clone(),
                witness_vec.clone(),
            )
                .prop_map(|(inputs, iv, key, outputs)| {
                    BlackBoxFuncCall::AES128Encrypt { inputs, iv, key, outputs }
                });

            let case_and = (input_arr_3.clone(), input_arr_8.clone(), witness.clone()).prop_map(
                |(lhs, rhs, output)| BlackBoxFuncCall::AND {
                    lhs: lhs[0],
                    rhs: rhs[1],
                    num_bits: 8,
                    output,
                },
            );

            let case_xor = (input_arr_3.clone(), input_arr_8.clone(), witness.clone()).prop_map(
                |(lhs, rhs, output)| BlackBoxFuncCall::XOR {
                    lhs: lhs[0],
                    rhs: rhs[1],
                    num_bits: 8,
                    output,
                },
            );

            let case_range = witness.clone().prop_map(|witness| BlackBoxFuncCall::RANGE {
                input: FunctionInput::Witness(witness),
                num_bits: 32,
            });

            let case_blake2s =
                (input_arr_8.clone(), witness_arr_32.clone()).prop_map(|(inputs, outputs)| {
                    BlackBoxFuncCall::Blake2s { inputs: inputs.to_vec(), outputs }
                });

            let case_blake3 =
                (input_arr_8.clone(), witness_arr_32.clone()).prop_map(|(inputs, outputs)| {
                    BlackBoxFuncCall::Blake3 { inputs: inputs.to_vec(), outputs }
                });

            let case_ecdsa_secp256k1 = (
                input_arr_32.clone(),
                input_arr_32.clone(),
                input_arr_64.clone(),
                input_arr_32.clone(),
                witness.clone(),
                input.clone(),
            )
                .prop_map(
                    |(public_key_x, public_key_y, signature, hashed_message, output, predicate)| {
                        BlackBoxFuncCall::EcdsaSecp256k1 {
                            public_key_x,
                            public_key_y,
                            signature,
                            hashed_message,
                            output,
                            predicate,
                        }
                    },
                );

            let case_ecdsa_secp256r1 = (
                input_arr_32.clone(),
                input_arr_32.clone(),
                input_arr_64.clone(),
                input_arr_32.clone(),
                witness.clone(),
                input.clone(),
            )
                .prop_map(
                    |(public_key_x, public_key_y, signature, hashed_message, output, predicate)| {
                        BlackBoxFuncCall::EcdsaSecp256r1 {
                            public_key_x,
                            public_key_y,
                            signature,
                            hashed_message,
                            output,
                            predicate,
                        }
                    },
                );

            let case_multi_scalar_mul = (
                input_vec.clone(),
                input_vec.clone(),
                input.clone(),
                witness.clone(),
                witness.clone(),
                witness.clone(),
            )
                .prop_map(|(points, scalars, predicate, w1, w2, w3)| {
                    BlackBoxFuncCall::MultiScalarMul {
                        points,
                        scalars,
                        predicate,
                        outputs: (w1, w2, w3),
                    }
                });

            let case_embedded_curve_add = (
                input_arr_3.clone(),
                input_arr_3.clone(),
                input.clone(),
                witness.clone(),
                witness.clone(),
                witness.clone(),
            )
                .prop_map(|(input1, input2, predicate, w1, w2, w3)| {
                    BlackBoxFuncCall::EmbeddedCurveAdd {
                        input1,
                        input2,
                        predicate,
                        outputs: (w1, w2, w3),
                    }
                });

            let case_keccakf1600 = (input_arr_25.clone(), witness_arr_25.clone())
                .prop_map(|(inputs, outputs)| BlackBoxFuncCall::Keccakf1600 { inputs, outputs });

            let case_recursive_aggregation = (
                input_vec.clone(),
                input_vec.clone(),
                input_vec.clone(),
                input.clone(),
                any::<u32>(),
                input.clone(),
                witness_vec.clone(),
            )
                .prop_map(
                    |(verification_key, proof, public_inputs, key_hash, proof_type, predicate, output)| {
                        BlackBoxFuncCall::RecursiveAggregation {
                            verification_key,
                            proof,
                            public_inputs,
                            key_hash,
                            proof_type,
                            predicate,
                            output,
                        }
                    },
                );

            let case_poseidon2_permutation =
                (input_vec.clone(), witness_vec.clone()).prop_map(|(inputs, outputs)| {
                    BlackBoxFuncCall::Poseidon2Permutation { inputs, outputs }
                });

            let case_sha256_compression = (input_arr_16, input_arr_8, witness_arr_8).prop_map(
                |(inputs, hash_values, outputs)| BlackBoxFuncCall::Sha256Compression {
                    inputs,
                    hash_values,
                    outputs,
                },
            );

            prop_oneof![
                case_aes128_encrypt,
                case_and,
                case_xor,
                case_range,
                case_blake2s,
                case_blake3,
                case_ecdsa_secp256k1,
                case_ecdsa_secp256r1,
                case_multi_scalar_mul,
                case_embedded_curve_add,
                case_keccakf1600,
                case_recursive_aggregation,
                case_poseidon2_permutation,
                case_sha256_compression,
            ]
            .boxed()
        }
    }
}
