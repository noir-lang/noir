use std::collections::BTreeSet;

use crate::native_types::Witness;
use crate::{AcirField, BlackBoxFunc};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

// Note: Some functions will not use all of the witness
// So we need to supply how many bits of the witness is needed

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ConstantOrWitnessEnum<F> {
    Constant(F),
    Witness(Witness),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct FunctionInput<F> {
    input: ConstantOrWitnessEnum<F>,
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

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum BlackBoxFuncCall<F> {
    AES128Encrypt {
        inputs: Vec<FunctionInput<F>>,
        iv: Box<[FunctionInput<F>; 16]>,
        key: Box<[FunctionInput<F>; 16]>,
        outputs: Vec<Witness>,
    },
    AND {
        lhs: FunctionInput<F>,
        rhs: FunctionInput<F>,
        output: Witness,
    },
    XOR {
        lhs: FunctionInput<F>,
        rhs: FunctionInput<F>,
        output: Witness,
    },
    RANGE {
        input: FunctionInput<F>,
    },
    Blake2s {
        inputs: Vec<FunctionInput<F>>,
        outputs: Box<[Witness; 32]>,
    },
    Blake3 {
        inputs: Vec<FunctionInput<F>>,
        outputs: Box<[Witness; 32]>,
    },
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
    MultiScalarMul {
        points: Vec<FunctionInput<F>>,
        scalars: Vec<FunctionInput<F>>,
        outputs: (Witness, Witness, Witness),
    },
    EmbeddedCurveAdd {
        input1: Box<[FunctionInput<F>; 3]>,
        input2: Box<[FunctionInput<F>; 3]>,
        outputs: (Witness, Witness, Witness),
    },
    Keccakf1600 {
        inputs: Box<[FunctionInput<F>; 25]>,
        outputs: Box<[Witness; 25]>,
    },
    RecursiveAggregation {
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
        proof_type: u32,
    },
    BigIntAdd {
        lhs: u32,
        rhs: u32,
        output: u32,
    },
    BigIntSub {
        lhs: u32,
        rhs: u32,
        output: u32,
    },
    BigIntMul {
        lhs: u32,
        rhs: u32,
        output: u32,
    },
    BigIntDiv {
        lhs: u32,
        rhs: u32,
        output: u32,
    },
    BigIntFromLeBytes {
        inputs: Vec<FunctionInput<F>>,
        modulus: Vec<u8>,
        output: u32,
    },
    BigIntToLeBytes {
        input: u32,
        outputs: Vec<Witness>,
    },
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

impl<F: Copy> BlackBoxFuncCall<F> {
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
