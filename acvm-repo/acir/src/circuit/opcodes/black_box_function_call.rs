use crate::native_types::Witness;
use crate::BlackBoxFunc;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

// Note: Some functions will not use all of the witness
// So we need to supply how many bits of the witness is needed
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionInput {
    pub witness: Witness,
    pub num_bits: u32,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlackBoxFuncCall {
    AES128Encrypt {
        inputs: Vec<FunctionInput>,
        iv: Box<[FunctionInput; 16]>,
        key: Box<[FunctionInput; 16]>,
        outputs: Vec<Witness>,
    },
    AND {
        lhs: FunctionInput,
        rhs: FunctionInput,
        output: Witness,
    },
    XOR {
        lhs: FunctionInput,
        rhs: FunctionInput,
        output: Witness,
    },
    RANGE {
        input: FunctionInput,
    },
    SHA256 {
        inputs: Vec<FunctionInput>,
        outputs: Box<[Witness; 32]>,
    },
    Blake2s {
        inputs: Vec<FunctionInput>,
        outputs: Box<[Witness; 32]>,
    },
    Blake3 {
        inputs: Vec<FunctionInput>,
        outputs: Box<[Witness; 32]>,
    },
    SchnorrVerify {
        public_key_x: FunctionInput,
        public_key_y: FunctionInput,
        #[serde(
            serialize_with = "serialize_big_array",
            deserialize_with = "deserialize_big_array_into_box"
        )]
        signature: Box<[FunctionInput; 64]>,
        message: Vec<FunctionInput>,
        output: Witness,
    },
    PedersenCommitment {
        inputs: Vec<FunctionInput>,
        domain_separator: u32,
        outputs: (Witness, Witness),
    },
    PedersenHash {
        inputs: Vec<FunctionInput>,
        domain_separator: u32,
        output: Witness,
    },
    EcdsaSecp256k1 {
        public_key_x: Box<[FunctionInput; 32]>,
        public_key_y: Box<[FunctionInput; 32]>,
        #[serde(
            serialize_with = "serialize_big_array",
            deserialize_with = "deserialize_big_array_into_box"
        )]
        signature: Box<[FunctionInput; 64]>,
        hashed_message: Box<[FunctionInput; 32]>,
        output: Witness,
    },
    EcdsaSecp256r1 {
        public_key_x: Box<[FunctionInput; 32]>,
        public_key_y: Box<[FunctionInput; 32]>,
        #[serde(
            serialize_with = "serialize_big_array",
            deserialize_with = "deserialize_big_array_into_box"
        )]
        signature: Box<[FunctionInput; 64]>,
        hashed_message: Box<[FunctionInput; 32]>,
        output: Witness,
    },
    MultiScalarMul {
        points: Vec<FunctionInput>,
        scalars: Vec<FunctionInput>,
        outputs: (Witness, Witness, Witness),
    },
    EmbeddedCurveAdd {
        input1: Box<[FunctionInput; 3]>,
        input2: Box<[FunctionInput; 3]>,
        outputs: (Witness, Witness, Witness),
    },
    Keccak256 {
        inputs: Vec<FunctionInput>,
        /// This is the number of bytes to take
        /// from the input. Note: if `var_message_size`
        /// is more than the number of bytes in the input,
        /// then an error is returned.
        var_message_size: FunctionInput,
        outputs: Box<[Witness; 32]>,
    },
    Keccakf1600 {
        inputs: Box<[FunctionInput; 25]>,
        outputs: Box<[Witness; 25]>,
    },
    RecursiveAggregation {
        verification_key: Vec<FunctionInput>,
        proof: Vec<FunctionInput>,
        /// These represent the public inputs of the proof we are verifying
        /// They should be checked against in the circuit after construction
        /// of a new aggregation state
        public_inputs: Vec<FunctionInput>,
        /// A key hash is used to check the validity of the verification key.
        /// The circuit implementing this opcode can use this hash to ensure that the
        /// key provided to the circuit matches the key produced by the circuit creator
        key_hash: FunctionInput,
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
        inputs: Vec<FunctionInput>,
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
        inputs: Vec<FunctionInput>,
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
        inputs: Box<[FunctionInput; 16]>,
        /// Vector of 8 u32s used to compress the input
        hash_values: Box<[FunctionInput; 8]>,
        /// Output of the compression, represented by 8 u32s
        outputs: Box<[Witness; 8]>,
    },
}

impl BlackBoxFuncCall {
    pub fn get_black_box_func(&self) -> BlackBoxFunc {
        match self {
            BlackBoxFuncCall::AES128Encrypt { .. } => BlackBoxFunc::AES128Encrypt,
            BlackBoxFuncCall::AND { .. } => BlackBoxFunc::AND,
            BlackBoxFuncCall::XOR { .. } => BlackBoxFunc::XOR,
            BlackBoxFuncCall::RANGE { .. } => BlackBoxFunc::RANGE,
            BlackBoxFuncCall::SHA256 { .. } => BlackBoxFunc::SHA256,
            BlackBoxFuncCall::Blake2s { .. } => BlackBoxFunc::Blake2s,
            BlackBoxFuncCall::Blake3 { .. } => BlackBoxFunc::Blake3,
            BlackBoxFuncCall::SchnorrVerify { .. } => BlackBoxFunc::SchnorrVerify,
            BlackBoxFuncCall::PedersenCommitment { .. } => BlackBoxFunc::PedersenCommitment,
            BlackBoxFuncCall::PedersenHash { .. } => BlackBoxFunc::PedersenHash,
            BlackBoxFuncCall::EcdsaSecp256k1 { .. } => BlackBoxFunc::EcdsaSecp256k1,
            BlackBoxFuncCall::EcdsaSecp256r1 { .. } => BlackBoxFunc::EcdsaSecp256r1,
            BlackBoxFuncCall::MultiScalarMul { .. } => BlackBoxFunc::MultiScalarMul,
            BlackBoxFuncCall::EmbeddedCurveAdd { .. } => BlackBoxFunc::EmbeddedCurveAdd,
            BlackBoxFuncCall::Keccak256 { .. } => BlackBoxFunc::Keccak256,
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

    pub fn get_inputs_vec(&self) -> Vec<FunctionInput> {
        match self {
            BlackBoxFuncCall::AES128Encrypt { inputs, .. }
            | BlackBoxFuncCall::SHA256 { inputs, .. }
            | BlackBoxFuncCall::Blake2s { inputs, .. }
            | BlackBoxFuncCall::Blake3 { inputs, .. }
            | BlackBoxFuncCall::PedersenCommitment { inputs, .. }
            | BlackBoxFuncCall::PedersenHash { inputs, .. }
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
                let mut inputs: Vec<FunctionInput> = Vec::with_capacity(points.len() * 2);
                inputs.extend(points.iter().copied());
                inputs.extend(scalars.iter().copied());
                inputs
            }
            BlackBoxFuncCall::EmbeddedCurveAdd { input1, input2, .. } => {
                vec![input1[0], input1[1], input2[0], input2[1]]
            }
            BlackBoxFuncCall::RANGE { input } => vec![*input],
            BlackBoxFuncCall::SchnorrVerify {
                public_key_x,
                public_key_y,
                signature,
                message,
                ..
            } => {
                let mut inputs: Vec<FunctionInput> =
                    Vec::with_capacity(2 + signature.len() + message.len());
                inputs.push(*public_key_x);
                inputs.push(*public_key_y);
                inputs.extend(signature.iter().copied());
                inputs.extend(message.iter().copied());
                inputs
            }
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
            BlackBoxFuncCall::Keccak256 { inputs, var_message_size, .. } => {
                let mut inputs = inputs.clone();
                inputs.push(*var_message_size);
                inputs
            }
            BlackBoxFuncCall::RecursiveAggregation {
                verification_key: key,
                proof,
                public_inputs,
                key_hash,
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
            BlackBoxFuncCall::SHA256 { outputs, .. }
            | BlackBoxFuncCall::Blake2s { outputs, .. }
            | BlackBoxFuncCall::Blake3 { outputs, .. }
            | BlackBoxFuncCall::Keccak256 { outputs, .. } => outputs.to_vec(),

            BlackBoxFuncCall::Keccakf1600 { outputs, .. } => outputs.to_vec(),

            BlackBoxFuncCall::Sha256Compression { outputs, .. } => outputs.to_vec(),

            BlackBoxFuncCall::AES128Encrypt { outputs, .. }
            | BlackBoxFuncCall::Poseidon2Permutation { outputs, .. } => outputs.to_vec(),

            BlackBoxFuncCall::AND { output, .. }
            | BlackBoxFuncCall::XOR { output, .. }
            | BlackBoxFuncCall::SchnorrVerify { output, .. }
            | BlackBoxFuncCall::EcdsaSecp256k1 { output, .. }
            | BlackBoxFuncCall::PedersenHash { output, .. }
            | BlackBoxFuncCall::EcdsaSecp256r1 { output, .. } => vec![*output],
            BlackBoxFuncCall::PedersenCommitment { outputs, .. } => vec![outputs.0, outputs.1],
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

const ABBREVIATION_LIMIT: usize = 5;

fn get_inputs_string(inputs: &[FunctionInput]) -> String {
    // Once a vectors length gets above this limit,
    // instead of listing all of their elements, we use ellipses
    // to abbreviate them
    let should_abbreviate_inputs = inputs.len() <= ABBREVIATION_LIMIT;

    if should_abbreviate_inputs {
        let mut result = String::new();
        for (index, inp) in inputs.iter().enumerate() {
            result += &format!("(_{}, num_bits: {})", inp.witness.witness_index(), inp.num_bits);
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

        result += &format!(
            "(_{}, num_bits: {})...(_{}, num_bits: {})",
            first.witness.witness_index(),
            first.num_bits,
            last.witness.witness_index(),
            last.num_bits,
        );

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

impl std::fmt::Display for BlackBoxFuncCall {
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

        // SPECIFIC PARAMETERS
        match self {
            BlackBoxFuncCall::PedersenCommitment { domain_separator, .. } => {
                write!(f, " domain_separator: {domain_separator}")
            }
            _ => write!(f, ""),
        }
    }
}

impl std::fmt::Debug for BlackBoxFuncCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

fn serialize_big_array<S>(big_array: &[FunctionInput; 64], s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use serde_big_array::BigArray;

    (*big_array).serialize(s)
}

fn deserialize_big_array_into_box<'de, D>(
    deserializer: D,
) -> Result<Box<[FunctionInput; 64]>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde_big_array::BigArray;

    let big_array: [FunctionInput; 64] = BigArray::deserialize(deserializer)?;
    Ok(Box::new(big_array))
}

#[cfg(test)]
mod tests {

    use crate::{circuit::Opcode, native_types::Witness};
    use acir_field::{AcirField, FieldElement};

    use super::{BlackBoxFuncCall, FunctionInput};

    fn keccakf1600_opcode<F: AcirField>() -> Opcode<F> {
        let inputs: Box<[FunctionInput; 25]> = Box::new(std::array::from_fn(|i| FunctionInput {
            witness: Witness(i as u32 + 1),
            num_bits: 8,
        }));
        let outputs: Box<[Witness; 25]> = Box::new(std::array::from_fn(|i| Witness(i as u32 + 26)));

        Opcode::BlackBoxFuncCall(BlackBoxFuncCall::Keccakf1600 { inputs, outputs })
    }
    fn schnorr_verify_opcode<F: AcirField>() -> Opcode<F> {
        let public_key_x =
            FunctionInput { witness: Witness(1), num_bits: FieldElement::max_num_bits() };
        let public_key_y =
            FunctionInput { witness: Witness(2), num_bits: FieldElement::max_num_bits() };
        let signature: Box<[FunctionInput; 64]> = Box::new(std::array::from_fn(|i| {
            FunctionInput { witness: Witness(i as u32 + 3), num_bits: 8 }
        }));
        let message: Vec<FunctionInput> = vec![FunctionInput { witness: Witness(67), num_bits: 8 }];
        let output = Witness(68);

        Opcode::BlackBoxFuncCall(BlackBoxFuncCall::SchnorrVerify {
            public_key_x,
            public_key_y,
            signature,
            message,
            output,
        })
    }

    #[test]
    fn keccakf1600_serialization_roundtrip() {
        let opcode = keccakf1600_opcode::<FieldElement>();
        let buf = bincode::serialize(&opcode).unwrap();
        let recovered_opcode = bincode::deserialize(&buf).unwrap();
        assert_eq!(opcode, recovered_opcode);
    }

    #[test]
    fn schnorr_serialization_roundtrip() {
        let opcode = schnorr_verify_opcode::<FieldElement>();
        let buf = bincode::serialize(&opcode).unwrap();
        let recovered_opcode = bincode::deserialize(&buf).unwrap();
        assert_eq!(opcode, recovered_opcode);
    }
}
