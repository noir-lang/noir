use serde::{Deserialize, Serialize};

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug, Hash, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OPCODE {
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
    ToBits,
}

impl std::fmt::Display for OPCODE {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl OPCODE {
    pub fn to_u16(self) -> u16 {
        match self {
            OPCODE::AES => 0,
            OPCODE::SHA256 => 1,
            OPCODE::MerkleMembership => 2,
            OPCODE::SchnorrVerify => 3,
            OPCODE::Blake2s => 4,
            OPCODE::Pedersen => 5,
            OPCODE::HashToField => 6,
            OPCODE::EcdsaSecp256k1 => 7,
            OPCODE::FixedBaseScalarMul => 8,
            OPCODE::InsertRegularMerkle => 9,
            OPCODE::ToBits => 10,
        }
    }
    pub fn name(&self) -> &str {
        match self {
            OPCODE::AES => "aes",
            OPCODE::SHA256 => "sha256",
            OPCODE::MerkleMembership => "merkle_membership",
            OPCODE::SchnorrVerify => "schnorr_verify",
            OPCODE::Blake2s => "blake2s",
            OPCODE::Pedersen => "pedersen",
            OPCODE::HashToField => "hash_to_field",
            OPCODE::EcdsaSecp256k1 => "ecdsa_secp256k1",
            OPCODE::FixedBaseScalarMul => "fixed_base_scalar_mul",
            OPCODE::InsertRegularMerkle => "insert_regular_merkle",
            OPCODE::ToBits => "to_bits",
        }
    }
    pub fn lookup(op_name: &str) -> Option<OPCODE> {
        match op_name {
            "sha256" => Some(OPCODE::SHA256),
            "merkle_membership" => Some(OPCODE::MerkleMembership),
            "schnorr_verify" => Some(OPCODE::SchnorrVerify),
            "blake2s" => Some(OPCODE::Blake2s),
            "pedersen" => Some(OPCODE::Pedersen),
            "hash_to_field" => Some(OPCODE::HashToField),
            "ecdsa_secp256k1" => Some(OPCODE::EcdsaSecp256k1),
            "fixed_base_scalar_mul" => Some(OPCODE::FixedBaseScalarMul),
            "insert_regular_merkle" => Some(OPCODE::InsertRegularMerkle),
            "to_bits" => Some(OPCODE::ToBits),
            _ => None,
        }
    }
    pub fn is_valid_opcode_name(op_name: &str) -> bool {
        OPCODE::lookup(op_name).is_some()
    }
    pub fn definition(&self) -> GadgetDefinition {
        match self {
            OPCODE::AES => unimplemented!(),
            OPCODE::SHA256 => GadgetDefinition {
                name: self.name().into(),
                input_size: InputSize::Variable,
                output_size: OutputSize(32),
            },
            OPCODE::Blake2s => GadgetDefinition {
                name: self.name().into(),
                input_size: InputSize::Variable,
                output_size: OutputSize(32),
            },
            OPCODE::HashToField => GadgetDefinition {
                name: self.name().into(),
                input_size: InputSize::Variable,
                output_size: OutputSize(1),
            },
            OPCODE::MerkleMembership => GadgetDefinition {
                name: self.name().into(),
                input_size: InputSize::Variable,
                output_size: OutputSize(1),
            },
            OPCODE::SchnorrVerify => GadgetDefinition {
                name: self.name().into(),
                // XXX: input_size can be changed to fixed, once we hash
                // the message before passing it to schnorr.
                // This is assuming all hashes will be 256 bits. Reasonable?
                input_size: InputSize::Variable,
                output_size: OutputSize(1),
            },
            OPCODE::Pedersen => GadgetDefinition {
                name: self.name().into(),
                input_size: InputSize::Variable,
                output_size: OutputSize(2),
            },
            OPCODE::EcdsaSecp256k1 => GadgetDefinition {
                name: self.name().into(),
                input_size: InputSize::Variable,
                output_size: OutputSize(1),
            },
            OPCODE::FixedBaseScalarMul => GadgetDefinition {
                name: self.name().into(),
                input_size: InputSize::Fixed(1),
                output_size: OutputSize(2),
            },
            OPCODE::InsertRegularMerkle => GadgetDefinition {
                name: self.name().into(),
                input_size: InputSize::Variable,
                output_size: OutputSize(1),
            },
            OPCODE::ToBits => GadgetDefinition {
                name: self.name().into(),
                input_size: InputSize::Fixed(2),
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
