//! Black box functions are ACIR opcodes which rely on backends implementing
//! support for specialized constraints.
//! This makes certain zk-snark unfriendly computations cheaper than if they were
//! implemented in more basic constraints.

use std::collections::BTreeSet;

use crate::native_types::Witness;
use crate::{AcirField, BlackBoxFunc};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

/// Enumeration for black box function inputs
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[cfg_attr(feature = "arb", derive(proptest_derive::Arbitrary))]
pub enum ConstantOrWitnessEnum<F> {
    /// A constant field element
    Constant(F),
    /// A witness element, representing dynamic inputs
    Witness(Witness),
}

/// Input to a black box call
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[cfg_attr(feature = "arb", derive(proptest_derive::Arbitrary))]
pub struct FunctionInput<F> {
    /// The actual input value
    input: ConstantOrWitnessEnum<F>,
    /// A constant representing the bit size of the input value
    /// Some functions will not use all of the witness
    /// So we need to supply how many bits of the witness is needed
    num_bits: u32,
}

impl<F> FunctionInput<F> {
    pub fn to_witness(&self) -> Witness {
        match self.input {
            ConstantOrWitnessEnum::Constant(_) => unreachable!("ICE - Expected Witness"),
            ConstantOrWitnessEnum::Witness(witness) => witness,
        }
    }

    pub fn input(self) -> ConstantOrWitnessEnum<F> {
        self.input
    }

    pub fn input_ref(&self) -> &ConstantOrWitnessEnum<F> {
        &self.input
    }

    pub fn num_bits(&self) -> u32 {
        self.num_bits
    }

    pub fn witness(witness: Witness, num_bits: u32) -> FunctionInput<F> {
        FunctionInput { input: ConstantOrWitnessEnum::Witness(witness), num_bits }
    }

    pub fn is_constant(&self) -> bool {
        matches!(self.input, ConstantOrWitnessEnum::Constant(_))
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Error)]
#[error("FunctionInput value has too many bits: value: {value}, {value_num_bits} >= {max_bits}")]
pub struct InvalidInputBitSize {
    pub value: String,
    pub value_num_bits: u32,
    pub max_bits: u32,
}

impl<F: AcirField> FunctionInput<F> {
    pub fn constant(value: F, max_bits: u32) -> Result<FunctionInput<F>, InvalidInputBitSize> {
        if value.num_bits() <= max_bits {
            Ok(FunctionInput { input: ConstantOrWitnessEnum::Constant(value), num_bits: max_bits })
        } else {
            let value_num_bits = value.num_bits();
            let value = format!("{}", value);
            Err(InvalidInputBitSize { value, value_num_bits, max_bits })
        }
    }
}

impl<F: std::fmt::Display> std::fmt::Display for FunctionInput<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.input {
            ConstantOrWitnessEnum::Constant(constant) => write!(f, "{constant}"),
            ConstantOrWitnessEnum::Witness(witness) => write!(f, "{}", witness.0),
        }
    }
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
    AND { lhs: FunctionInput<F>, rhs: FunctionInput<F>, output: Witness },
    /// Performs the bitwise XOR of `lhs` and `rhs`. `bit_size` must be the same for
    /// both inputs.
    /// - lhs: (witness, bit_size)
    /// - rhs: (witness, bit_size)
    /// - output: a witness whose value is constrained to be lhs XOR rhs, as
    ///   bit_size bit integers
    XOR { lhs: FunctionInput<F>, rhs: FunctionInput<F>, output: Witness },
    /// Range constraint to ensure that a witness
    /// can be represented in the specified number of bits.
    /// - input: (witness, bit_size)
    RANGE { input: FunctionInput<F> },
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
    EcdsaSecp256k1 {
        public_key_x: Box<[FunctionInput<F>; 32]>,
        public_key_y: Box<[FunctionInput<F>; 32]>,
        #[serde(
            serialize_with = "serialize_big_array",
            deserialize_with = "deserialize_big_array_into_box"
        )]
        signature: Box<[FunctionInput<F>; 64]>,
        hashed_message: Box<[FunctionInput<F>; 32]>,
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
        output: Witness,
    },
    /// Multiple scalar multiplication (MSM) with a variable base/input point
    /// (P) of the embedded curve. An MSM multiplies the points and scalars and
    /// sums the results.
    /// - input:
    ///     points (witness, N) a vector of x and y coordinates of input
    ///     points `[x1, y1, x2, y2,...]`.
    ///     scalars (witness, N) a vector of low and high limbs of input
    ///     scalars `[s1_low, s1_high, s2_low, s2_high, ...]`. (witness, N)
    ///     For Barretenberg, they must both be less than 128 bits.
    /// - output:
    ///     a tuple of `x` and `y` coordinates of output.
    ///     Points computed as `s_low*P+s_high*2^{128}*P`
    ///
    /// Because the Grumpkin scalar field is bigger than the ACIR field, we
    /// provide 2 ACIR fields representing the low and high parts of the Grumpkin
    /// scalar $a$: `a=low+high*2^{128}`, with `low, high < 2^{128}`
    MultiScalarMul {
        points: Vec<FunctionInput<F>>,
        scalars: Vec<FunctionInput<F>>,
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
    },
    /// BigInt addition
    BigIntAdd { lhs: u32, rhs: u32, output: u32 },
    /// BigInt subtraction
    BigIntSub { lhs: u32, rhs: u32, output: u32 },
    /// BigInt multiplication
    BigIntMul { lhs: u32, rhs: u32, output: u32 },
    /// BigInt division
    BigIntDiv { lhs: u32, rhs: u32, output: u32 },
    /// BigInt from le bytes
    BigIntFromLeBytes { inputs: Vec<FunctionInput<F>>, modulus: Vec<u8>, output: u32 },
    /// BigInt to le bytes
    BigIntToLeBytes { input: u32, outputs: Vec<Witness> },
    /// Applies the Poseidon2 permutation function to the given state,
    /// outputting the permuted state.
    Poseidon2Permutation {
        /// Input state for the permutation of Poseidon2
        inputs: Vec<FunctionInput<F>>,
        /// Permuted state
        outputs: Vec<Witness>,
        /// State length (in number of field elements)
        /// It is the length of inputs and outputs vectors
        len: u32,
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
            BlackBoxFuncCall::BigIntAdd { .. } => BlackBoxFunc::BigIntAdd,
            BlackBoxFuncCall::BigIntSub { .. } => BlackBoxFunc::BigIntSub,
            BlackBoxFuncCall::BigIntMul { .. } => BlackBoxFunc::BigIntMul,
            BlackBoxFuncCall::BigIntDiv { .. } => BlackBoxFunc::BigIntDiv,
            BlackBoxFuncCall::BigIntFromLeBytes { .. } => BlackBoxFunc::BigIntFromLeBytes,
            BlackBoxFuncCall::BigIntToLeBytes { .. } => BlackBoxFunc::BigIntToLeBytes,
            BlackBoxFuncCall::Poseidon2Permutation { .. } => BlackBoxFunc::Poseidon2Permutation,
            BlackBoxFuncCall::Sha256Compression { .. } => BlackBoxFunc::Sha256Compression,
        }
    }

    pub fn name(&self) -> &str {
        self.get_black_box_func().name()
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
            BlackBoxFuncCall::RANGE { .. }
            | BlackBoxFuncCall::RecursiveAggregation { .. }
            | BlackBoxFuncCall::BigIntFromLeBytes { .. }
            | BlackBoxFuncCall::BigIntAdd { .. }
            | BlackBoxFuncCall::BigIntSub { .. }
            | BlackBoxFuncCall::BigIntMul { .. }
            | BlackBoxFuncCall::BigIntDiv { .. } => {
                vec![]
            }
            BlackBoxFuncCall::BigIntToLeBytes { outputs, .. } => outputs.to_vec(),
        }
    }
}

impl<F: Copy> BlackBoxFuncCall<F> {
    pub fn get_inputs_vec(&self) -> Vec<FunctionInput<F>> {
        match self {
            BlackBoxFuncCall::AES128Encrypt { inputs, .. }
            | BlackBoxFuncCall::Blake2s { inputs, .. }
            | BlackBoxFuncCall::Blake3 { inputs, .. }
            | BlackBoxFuncCall::BigIntFromLeBytes { inputs, .. }
            | BlackBoxFuncCall::Poseidon2Permutation { inputs, .. } => inputs.to_vec(),

            BlackBoxFuncCall::Keccakf1600 { inputs, .. } => inputs.to_vec(),

            BlackBoxFuncCall::Sha256Compression { inputs, hash_values, .. } => {
                inputs.iter().chain(hash_values.as_ref()).copied().collect()
            }
            BlackBoxFuncCall::AND { lhs, rhs, .. } | BlackBoxFuncCall::XOR { lhs, rhs, .. } => {
                vec![*lhs, *rhs]
            }
            BlackBoxFuncCall::BigIntAdd { .. }
            | BlackBoxFuncCall::BigIntSub { .. }
            | BlackBoxFuncCall::BigIntMul { .. }
            | BlackBoxFuncCall::BigIntDiv { .. }
            | BlackBoxFuncCall::BigIntToLeBytes { .. } => Vec::new(),
            BlackBoxFuncCall::MultiScalarMul { points, scalars, .. } => {
                let mut inputs: Vec<FunctionInput<F>> =
                    Vec::with_capacity(points.len() + scalars.len());
                inputs.extend(points.iter().copied());
                inputs.extend(scalars.iter().copied());
                inputs
            }
            BlackBoxFuncCall::EmbeddedCurveAdd { input1, input2, .. } => {
                vec![input1[0], input1[1], input2[0], input2[1]]
            }
            BlackBoxFuncCall::RANGE { input } => vec![*input],
            BlackBoxFuncCall::EcdsaSecp256k1 {
                public_key_x,
                public_key_y,
                signature,
                hashed_message,
                ..
            } => {
                let mut inputs = Vec::with_capacity(
                    public_key_x.len()
                        + public_key_y.len()
                        + signature.len()
                        + hashed_message.len(),
                );
                inputs.extend(public_key_x.iter().copied());
                inputs.extend(public_key_y.iter().copied());
                inputs.extend(signature.iter().copied());
                inputs.extend(hashed_message.iter().copied());
                inputs
            }
            BlackBoxFuncCall::EcdsaSecp256r1 {
                public_key_x,
                public_key_y,
                signature,
                hashed_message,
                ..
            } => {
                let mut inputs = Vec::with_capacity(
                    public_key_x.len()
                        + public_key_y.len()
                        + signature.len()
                        + hashed_message.len(),
                );
                inputs.extend(public_key_x.iter().copied());
                inputs.extend(public_key_y.iter().copied());
                inputs.extend(signature.iter().copied());
                inputs.extend(hashed_message.iter().copied());
                inputs
            }
            BlackBoxFuncCall::RecursiveAggregation {
                verification_key: key,
                proof,
                public_inputs,
                key_hash,
                proof_type: _,
            } => {
                let mut inputs = Vec::new();
                inputs.extend(key.iter().copied());
                inputs.extend(proof.iter().copied());
                inputs.extend(public_inputs.iter().copied());
                inputs.push(*key_hash);
                inputs
            }
        }
    }

    pub fn get_input_witnesses(&self) -> BTreeSet<Witness> {
        let mut result = BTreeSet::new();
        for input in self.get_inputs_vec() {
            if let ConstantOrWitnessEnum::Witness(w) = input.input() {
                result.insert(w);
            }
        }
        result
    }
}

const ABBREVIATION_LIMIT: usize = 5;

fn get_inputs_string<F: std::fmt::Display>(inputs: &[FunctionInput<F>]) -> String {
    // Once a vectors length gets above this limit,
    // instead of listing all of their elements, we use ellipses
    // to abbreviate them
    let should_abbreviate_inputs = inputs.len() <= ABBREVIATION_LIMIT;

    if should_abbreviate_inputs {
        let mut result = String::new();
        for (index, inp) in inputs.iter().enumerate() {
            result += &format!("({})", inp);
            // Add a comma, unless it is the last entry
            if index != inputs.len() - 1 {
                result += ", ";
            }
        }
        result
    } else {
        let first = inputs.first().unwrap();
        let last = inputs.last().unwrap();

        let mut result = String::new();
        result += &format!("({})...({})", first, last,);

        result
    }
}

fn get_outputs_string(outputs: &[Witness]) -> String {
    let should_abbreviate_outputs = outputs.len() <= ABBREVIATION_LIMIT;

    if should_abbreviate_outputs {
        let mut result = String::new();
        for (index, output) in outputs.iter().enumerate() {
            result += &format!("_{}", output.witness_index());
            // Add a comma, unless it is the last entry
            if index != outputs.len() - 1 {
                result += ", ";
            }
        }
        result
    } else {
        let first = outputs.first().unwrap();
        let last = outputs.last().unwrap();

        let mut result = String::new();
        result += &format!("(_{},...,_{})", first.witness_index(), last.witness_index());
        result
    }
}

impl<F: std::fmt::Display + Copy> std::fmt::Display for BlackBoxFuncCall<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let uppercase_name = self.name().to_uppercase();
        write!(f, "BLACKBOX::{uppercase_name} ")?;
        // INPUTS
        write!(f, "[")?;

        let inputs_str = get_inputs_string(&self.get_inputs_vec());

        write!(f, "{inputs_str}")?;
        write!(f, "] ")?;

        // OUTPUTS
        write!(f, "[ ")?;

        let outputs_str = get_outputs_string(&self.get_outputs_vec());

        write!(f, "{outputs_str}")?;

        write!(f, "]")?;

        write!(f, "")
    }
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
            Box::new(std::array::from_fn(|i| FunctionInput::witness(Witness(i as u32 + 1), 8)));
        let outputs: Box<[Witness; 25]> = Box::new(std::array::from_fn(|i| Witness(i as u32 + 26)));

        Opcode::BlackBoxFuncCall(BlackBoxFuncCall::Keccakf1600 { inputs, outputs })
    }

    #[test]
    fn keccakf1600_serialization_roundtrip() {
        let opcode = keccakf1600_opcode::<FieldElement>();
        let buf = bincode::serialize(&opcode).unwrap();
        let recovered_opcode = bincode::deserialize(&buf).unwrap();
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

            let case_and = (input.clone(), input.clone(), witness.clone())
                .prop_map(|(lhs, rhs, output)| BlackBoxFuncCall::AND { lhs, rhs, output });

            let case_xor = (input.clone(), input.clone(), witness.clone())
                .prop_map(|(lhs, rhs, output)| BlackBoxFuncCall::XOR { lhs, rhs, output });

            let case_range = input.clone().prop_map(|input| BlackBoxFuncCall::RANGE { input });

            let case_blake2s = (input_vec.clone(), witness_arr_32.clone())
                .prop_map(|(inputs, outputs)| BlackBoxFuncCall::Blake2s { inputs, outputs });

            let case_blake3 = (input_vec.clone(), witness_arr_32.clone())
                .prop_map(|(inputs, outputs)| BlackBoxFuncCall::Blake3 { inputs, outputs });

            let case_ecdsa_secp256k1 = (
                input_arr_32.clone(),
                input_arr_32.clone(),
                input_arr_64.clone(),
                input_arr_32.clone(),
                witness.clone(),
            )
                .prop_map(
                    |(public_key_x, public_key_y, signature, hashed_message, output)| {
                        BlackBoxFuncCall::EcdsaSecp256k1 {
                            public_key_x,
                            public_key_y,
                            signature,
                            hashed_message,
                            output,
                        }
                    },
                );

            let case_ecdsa_secp256r1 = (
                input_arr_32.clone(),
                input_arr_32.clone(),
                input_arr_64.clone(),
                input_arr_32.clone(),
                witness.clone(),
            )
                .prop_map(
                    |(public_key_x, public_key_y, signature, hashed_message, output)| {
                        BlackBoxFuncCall::EcdsaSecp256r1 {
                            public_key_x,
                            public_key_y,
                            signature,
                            hashed_message,
                            output,
                        }
                    },
                );

            let case_multi_scalar_mul = (
                input_vec.clone(),
                input_vec.clone(),
                witness.clone(),
                witness.clone(),
                witness.clone(),
            )
                .prop_map(|(points, scalars, w1, w2, w3)| {
                    BlackBoxFuncCall::MultiScalarMul { points, scalars, outputs: (w1, w2, w3) }
                });

            let case_embedded_curve_add = (
                input_arr_3.clone(),
                input_arr_3.clone(),
                witness.clone(),
                witness.clone(),
                witness.clone(),
            )
                .prop_map(|(input1, input2, w1, w2, w3)| {
                    BlackBoxFuncCall::EmbeddedCurveAdd { input1, input2, outputs: (w1, w2, w3) }
                });

            let case_keccakf1600 = (input_arr_25.clone(), witness_arr_25.clone())
                .prop_map(|(inputs, outputs)| BlackBoxFuncCall::Keccakf1600 { inputs, outputs });

            let case_recursive_aggregation = (
                input_vec.clone(),
                input_vec.clone(),
                input_vec.clone(),
                input.clone(),
                any::<u32>(),
            )
                .prop_map(
                    |(verification_key, proof, public_inputs, key_hash, proof_type)| {
                        BlackBoxFuncCall::RecursiveAggregation {
                            verification_key,
                            proof,
                            public_inputs,
                            key_hash,
                            proof_type,
                        }
                    },
                );

            let big_int_args = (any::<u32>(), any::<u32>(), any::<u32>());

            let case_big_int_add = big_int_args
                .prop_map(|(lhs, rhs, output)| BlackBoxFuncCall::BigIntAdd { lhs, rhs, output });

            let case_big_int_sub = big_int_args
                .prop_map(|(lhs, rhs, output)| BlackBoxFuncCall::BigIntSub { lhs, rhs, output });

            let case_big_int_mul = big_int_args
                .prop_map(|(lhs, rhs, output)| BlackBoxFuncCall::BigIntMul { lhs, rhs, output });

            let case_big_int_div = big_int_args
                .prop_map(|(lhs, rhs, output)| BlackBoxFuncCall::BigIntDiv { lhs, rhs, output });

            let case_big_int_from_le_bytes = (input_vec.clone(), any::<Vec<u8>>(), any::<u32>())
                .prop_map(|(inputs, modulus, output)| BlackBoxFuncCall::BigIntFromLeBytes {
                    inputs,
                    modulus,
                    output,
                });

            let case_big_int_to_le_bytes = (any::<u32>(), witness_vec.clone())
                .prop_map(|(input, outputs)| BlackBoxFuncCall::BigIntToLeBytes { input, outputs });

            let case_poseidon2_permutation = (input_vec.clone(), witness_vec.clone(), any::<u32>())
                .prop_map(|(inputs, outputs, len)| BlackBoxFuncCall::Poseidon2Permutation {
                    inputs,
                    outputs,
                    len,
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
                case_big_int_add,
                case_big_int_sub,
                case_big_int_mul,
                case_big_int_div,
                case_big_int_from_le_bytes,
                case_big_int_to_le_bytes,
                case_poseidon2_permutation,
                case_sha256_compression,
            ]
            .boxed()
        }
    }
}
