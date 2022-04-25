use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Hash, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpCode {
    #[allow(clippy::upper_case_acronyms)]
    AES,
    SHA256,
    Blake2s,
    MerkleMembership,
    InsertRegularMerkle,
    SchnorrVerify,
    Pedersen,
    HashToField,
    EcdsaSecp256k1,
    FixedBaseScalarMul,
}

impl std::fmt::Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl OpCode {
    pub fn to_u16(self) -> u16 {
        match self {
            OpCode::AES => 0,
            OpCode::SHA256 => 1,
            OpCode::MerkleMembership => 2,
            OpCode::SchnorrVerify => 3,
            OpCode::Blake2s => 4,
            OpCode::Pedersen => 5,
            OpCode::HashToField => 6,
            OpCode::EcdsaSecp256k1 => 7,
            OpCode::FixedBaseScalarMul => 8,
            OpCode::InsertRegularMerkle => 9,
        }
    }
    pub fn name(&self) -> &str {
        match self {
            OpCode::AES => "aes",
            OpCode::SHA256 => "sha256",
            OpCode::MerkleMembership => "merkle_membership",
            OpCode::SchnorrVerify => "schnorr_verify",
            OpCode::Blake2s => "blake2s",
            OpCode::Pedersen => "pedersen",
            OpCode::HashToField => "hash_to_field",
            OpCode::EcdsaSecp256k1 => "ecdsa_secp256k1",
            OpCode::FixedBaseScalarMul => "fixed_base_scalar_mul",
            OpCode::InsertRegularMerkle => "insert_regular_merkle",
        }
    }
    pub fn lookup(op_name: &str) -> Option<OpCode> {
        match op_name {
            "sha256" => Some(OpCode::SHA256),
            "merkle_membership" => Some(OpCode::MerkleMembership),
            "schnorr_verify" => Some(OpCode::SchnorrVerify),
            "blake2s" => Some(OpCode::Blake2s),
            "pedersen" => Some(OpCode::Pedersen),
            "hash_to_field" => Some(OpCode::HashToField),
            "ecdsa_secp256k1" => Some(OpCode::EcdsaSecp256k1),
            "fixed_base_scalar_mul" => Some(OpCode::FixedBaseScalarMul),
            "insert_regular_merkle" => Some(OpCode::InsertRegularMerkle),
            _ => None,
        }
    }
    pub fn is_valid_opcode_name(op_name: &str) -> bool {
        OpCode::lookup(op_name).is_some()
    }
    pub fn definition(&self) -> GadgetDefinition {
        match self {
            OpCode::AES => unimplemented!(),
            OpCode::SHA256 => GadgetDefinition {
                name: self.name().into(),
                input_size: InputSize::Variable,
                output_size: OutputSize(32),
            },
            OpCode::Blake2s => GadgetDefinition {
                name: self.name().into(),
                input_size: InputSize::Variable,
                output_size: OutputSize(32),
            },
            OpCode::HashToField => GadgetDefinition {
                name: self.name().into(),
                input_size: InputSize::Variable,
                output_size: OutputSize(1),
            },
            OpCode::MerkleMembership => GadgetDefinition {
                name: self.name().into(),
                input_size: InputSize::Variable,
                output_size: OutputSize(1),
            },
            OpCode::SchnorrVerify => GadgetDefinition {
                name: self.name().into(),
                // XXX: input_size can be changed to fixed, once we hash
                // the message before passing it to schnorr.
                // This is assuming all hashes will be 256 bits. Reasonable?
                input_size: InputSize::Variable,
                output_size: OutputSize(1),
            },
            OpCode::Pedersen => GadgetDefinition {
                name: self.name().into(),
                input_size: InputSize::Variable,
                output_size: OutputSize(2),
            },
            OpCode::EcdsaSecp256k1 => GadgetDefinition {
                name: self.name().into(),
                input_size: InputSize::Variable,
                output_size: OutputSize(1),
            },
            OpCode::FixedBaseScalarMul => GadgetDefinition {
                name: self.name().into(),
                input_size: InputSize::Fixed(1),
                output_size: OutputSize(2),
            },
            OpCode::InsertRegularMerkle => GadgetDefinition {
                name: self.name().into(),
                input_size: InputSize::Variable,
                output_size: OutputSize(1),
            },
        }
    }
}

// Descriptor as to whether the input/output is fixed or variable
// Example: The input for Sha256 is Variable and the output is fixed at 2 witnesses
// each holding 128 bits of the actual Sha256 function
#[derive(Clone, Debug, Hash, PartialEq)]
pub enum InputSize {
    Variable,
    Fixed(u128),
}

// Output size Cannot currently vary, so we use a separate struct
// XXX: In the future, we may be able to allow the output to vary based on the input size, however this implies support for dynamic circuits
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct OutputSize(pub u128);

#[derive(Clone, Debug, Hash)]
// Specs for how many inputs/outputs the method takes.
pub struct GadgetDefinition {
    pub name: String,
    pub input_size: InputSize,
    pub output_size: OutputSize,
}
