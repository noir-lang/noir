use crate::native_types::Witness;
use crate::BlackBoxFunc;
use serde::{Deserialize, Serialize};

// Note: Some functions will not use all of the witness
// So we need to supply how many bits of the witness is needed
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionInput {
    pub witness: Witness,
    pub num_bits: u32,
}

impl FunctionInput {
    pub fn dummy() -> Self {
        Self { witness: Witness(0), num_bits: 0 }
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlackBoxFuncCall {
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
        outputs: Vec<Witness>,
    },
    Blake2s {
        inputs: Vec<FunctionInput>,
        outputs: Vec<Witness>,
    },
    SchnorrVerify {
        public_key_x: FunctionInput,
        public_key_y: FunctionInput,
        signature: Vec<FunctionInput>,
        message: Vec<FunctionInput>,
        output: Witness,
    },
    Pedersen {
        inputs: Vec<FunctionInput>,
        domain_separator: u32,
        outputs: (Witness, Witness),
    },
    // 128 here specifies that this function
    // should have 128 bits of security
    HashToField128Security {
        inputs: Vec<FunctionInput>,
        output: Witness,
    },
    EcdsaSecp256k1 {
        public_key_x: Vec<FunctionInput>,
        public_key_y: Vec<FunctionInput>,
        signature: Vec<FunctionInput>,
        hashed_message: Vec<FunctionInput>,
        output: Witness,
    },
    EcdsaSecp256r1 {
        public_key_x: Vec<FunctionInput>,
        public_key_y: Vec<FunctionInput>,
        signature: Vec<FunctionInput>,
        hashed_message: Vec<FunctionInput>,
        output: Witness,
    },
    FixedBaseScalarMul {
        low: FunctionInput,
        high: FunctionInput,
        outputs: (Witness, Witness),
    },
    Keccak256 {
        inputs: Vec<FunctionInput>,
        outputs: Vec<Witness>,
    },
    Keccak256VariableLength {
        inputs: Vec<FunctionInput>,
        /// This is the number of bytes to take
        /// from the input. Note: if `var_message_size`
        /// is more than the number of bytes in the input,
        /// then an error is returned.
        var_message_size: FunctionInput,
        outputs: Vec<Witness>,
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
        /// An aggregation object is blob of data that the top-level verifier must run some proof system specific
        /// algorithm on to complete verification. The size is proof system specific and will be set by the backend integrating this opcode.
        /// The input aggregation object is only not `None` when we are verifying a previous recursive aggregation in
        /// the current circuit. If this is the first recursive aggregation there is no input aggregation object.
        /// It is left to the backend to determine how to handle when there is no input aggregation object.
        input_aggregation_object: Option<Vec<FunctionInput>>,
        /// This is the result of a recursive aggregation and is what will be fed into the next verifier.
        /// The next verifier can either perform a final verification (returning true or false)
        /// or perform another recursive aggregation where this output aggregation object
        /// will be the input aggregation object of the next recursive aggregation.
        output_aggregation_object: Vec<Witness>,
    },
}

impl BlackBoxFuncCall {
    #[deprecated = "BlackBoxFuncCall::dummy() is unnecessary and will be removed in ACVM 0.24.0"]
    pub fn dummy(bb_func: BlackBoxFunc) -> Self {
        match bb_func {
            BlackBoxFunc::AND => BlackBoxFuncCall::AND {
                lhs: FunctionInput::dummy(),
                rhs: FunctionInput::dummy(),
                output: Witness(0),
            },
            BlackBoxFunc::XOR => BlackBoxFuncCall::XOR {
                lhs: FunctionInput::dummy(),
                rhs: FunctionInput::dummy(),
                output: Witness(0),
            },
            BlackBoxFunc::RANGE => BlackBoxFuncCall::RANGE { input: FunctionInput::dummy() },
            BlackBoxFunc::SHA256 => BlackBoxFuncCall::SHA256 { inputs: vec![], outputs: vec![] },
            BlackBoxFunc::Blake2s => BlackBoxFuncCall::Blake2s { inputs: vec![], outputs: vec![] },
            BlackBoxFunc::SchnorrVerify => BlackBoxFuncCall::SchnorrVerify {
                public_key_x: FunctionInput::dummy(),
                public_key_y: FunctionInput::dummy(),
                signature: vec![],
                message: vec![],
                output: Witness(0),
            },
            BlackBoxFunc::Pedersen => BlackBoxFuncCall::Pedersen {
                inputs: vec![],
                domain_separator: 0,
                outputs: (Witness(0), Witness(0)),
            },
            BlackBoxFunc::HashToField128Security => {
                BlackBoxFuncCall::HashToField128Security { inputs: vec![], output: Witness(0) }
            }
            BlackBoxFunc::EcdsaSecp256k1 => BlackBoxFuncCall::EcdsaSecp256k1 {
                public_key_x: vec![],
                public_key_y: vec![],
                signature: vec![],
                hashed_message: vec![],
                output: Witness(0),
            },
            BlackBoxFunc::EcdsaSecp256r1 => BlackBoxFuncCall::EcdsaSecp256r1 {
                public_key_x: vec![],
                public_key_y: vec![],
                signature: vec![],
                hashed_message: vec![],
                output: Witness(0),
            },
            BlackBoxFunc::FixedBaseScalarMul => BlackBoxFuncCall::FixedBaseScalarMul {
                low: FunctionInput::dummy(),
                high: FunctionInput::dummy(),
                outputs: (Witness(0), Witness(0)),
            },
            BlackBoxFunc::Keccak256 => {
                BlackBoxFuncCall::Keccak256 { inputs: vec![], outputs: vec![] }
            }
            BlackBoxFunc::RecursiveAggregation => BlackBoxFuncCall::RecursiveAggregation {
                verification_key: vec![],
                proof: vec![],
                public_inputs: vec![],
                key_hash: FunctionInput::dummy(),
                input_aggregation_object: None,
                output_aggregation_object: vec![],
            },
        }
    }

    pub fn get_black_box_func(&self) -> BlackBoxFunc {
        match self {
            BlackBoxFuncCall::AND { .. } => BlackBoxFunc::AND,
            BlackBoxFuncCall::XOR { .. } => BlackBoxFunc::XOR,
            BlackBoxFuncCall::RANGE { .. } => BlackBoxFunc::RANGE,
            BlackBoxFuncCall::SHA256 { .. } => BlackBoxFunc::SHA256,
            BlackBoxFuncCall::Blake2s { .. } => BlackBoxFunc::Blake2s,
            BlackBoxFuncCall::SchnorrVerify { .. } => BlackBoxFunc::SchnorrVerify,
            BlackBoxFuncCall::Pedersen { .. } => BlackBoxFunc::Pedersen,
            BlackBoxFuncCall::HashToField128Security { .. } => BlackBoxFunc::HashToField128Security,
            BlackBoxFuncCall::EcdsaSecp256k1 { .. } => BlackBoxFunc::EcdsaSecp256k1,
            BlackBoxFuncCall::EcdsaSecp256r1 { .. } => BlackBoxFunc::EcdsaSecp256r1,
            BlackBoxFuncCall::FixedBaseScalarMul { .. } => BlackBoxFunc::FixedBaseScalarMul,
            BlackBoxFuncCall::Keccak256 { .. } => BlackBoxFunc::Keccak256,
            BlackBoxFuncCall::Keccak256VariableLength { .. } => BlackBoxFunc::Keccak256,
            BlackBoxFuncCall::RecursiveAggregation { .. } => BlackBoxFunc::RecursiveAggregation,
        }
    }

    pub fn name(&self) -> &str {
        self.get_black_box_func().name()
    }

    pub fn get_inputs_vec(&self) -> Vec<FunctionInput> {
        match self {
            BlackBoxFuncCall::SHA256 { inputs, .. }
            | BlackBoxFuncCall::Blake2s { inputs, .. }
            | BlackBoxFuncCall::Keccak256 { inputs, .. }
            | BlackBoxFuncCall::Pedersen { inputs, .. }
            | BlackBoxFuncCall::HashToField128Security { inputs, .. } => inputs.to_vec(),
            BlackBoxFuncCall::AND { lhs, rhs, .. } | BlackBoxFuncCall::XOR { lhs, rhs, .. } => {
                vec![*lhs, *rhs]
            }
            BlackBoxFuncCall::FixedBaseScalarMul { low, high, .. } => vec![*low, *high],
            BlackBoxFuncCall::RANGE { input } => vec![*input],
            BlackBoxFuncCall::SchnorrVerify {
                public_key_x,
                public_key_y,
                signature,
                message,
                ..
            } => {
                let mut inputs = Vec::with_capacity(2 + signature.len() + message.len());
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
            BlackBoxFuncCall::Keccak256VariableLength { inputs, var_message_size, .. } => {
                let mut inputs = inputs.clone();
                inputs.push(*var_message_size);
                inputs
            }
            BlackBoxFuncCall::RecursiveAggregation {
                verification_key: key,
                proof,
                public_inputs,
                key_hash,
                ..
            } => {
                let mut inputs = Vec::new();
                inputs.extend(key.iter().copied());
                inputs.extend(proof.iter().copied());
                inputs.extend(public_inputs.iter().copied());
                inputs.push(*key_hash);
                // NOTE: we do not return an input aggregation object as it will either be non-existent for the first recursive aggregation
                // or the output aggregation object of a previous recursive aggregation. We do not simulate recursive aggregation
                // thus the input aggregation object will always be unassigned until proving
                inputs
            }
        }
    }

    pub fn get_outputs_vec(&self) -> Vec<Witness> {
        match self {
            BlackBoxFuncCall::SHA256 { outputs, .. }
            | BlackBoxFuncCall::Blake2s { outputs, .. }
            | BlackBoxFuncCall::Keccak256 { outputs, .. }
            | BlackBoxFuncCall::RecursiveAggregation {
                output_aggregation_object: outputs, ..
            } => outputs.to_vec(),
            BlackBoxFuncCall::AND { output, .. }
            | BlackBoxFuncCall::XOR { output, .. }
            | BlackBoxFuncCall::HashToField128Security { output, .. }
            | BlackBoxFuncCall::SchnorrVerify { output, .. }
            | BlackBoxFuncCall::EcdsaSecp256k1 { output, .. }
            | BlackBoxFuncCall::EcdsaSecp256r1 { output, .. } => vec![*output],
            BlackBoxFuncCall::FixedBaseScalarMul { outputs, .. }
            | BlackBoxFuncCall::Pedersen { outputs, .. } => vec![outputs.0, outputs.1],
            BlackBoxFuncCall::RANGE { .. } => vec![],
            BlackBoxFuncCall::Keccak256VariableLength { outputs, .. } => outputs.to_vec(),
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
            BlackBoxFuncCall::Pedersen { domain_separator, .. } => {
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
