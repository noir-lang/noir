use acvm::{acir::BlackBoxFunc, FieldElement};
use num_bigint::BigUint;
use num_traits::{One, Zero};

use super::node::ObjectType;

#[derive(Clone, Debug, Hash, Copy, PartialEq, Eq)]
pub enum Opcode {
    LowLevel(BlackBoxFunc),
    ToBits,
    ToRadix,
}

impl std::fmt::Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Opcode {
    pub fn lookup(op_name: &str) -> Option<Opcode> {
        match op_name {
            "to_bits_le" => Some(Opcode::ToBits),
            "to_radix" => Some(Opcode::ToRadix),
            _ => BlackBoxFunc::lookup(op_name).map(Opcode::LowLevel),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Opcode::LowLevel(op) => op.name(),
            Opcode::ToBits => "to_bits_le",
            Opcode::ToRadix => "to_radix",
        }
    }

    pub fn get_max_value(&self) -> BigUint {
        match self {
            Opcode::LowLevel(op) => {
                match op {
                    BlackBoxFunc::SHA256
                    | BlackBoxFunc::Blake2s
                    | BlackBoxFunc::Pedersen
                    | BlackBoxFunc::FixedBaseScalarMul => BigUint::zero(), //pointers do not overflow
                    BlackBoxFunc::SchnorrVerify
                    | BlackBoxFunc::EcdsaSecp256k1
                    | BlackBoxFunc::MerkleMembership => BigUint::one(), //verify returns 0 or 1
                    BlackBoxFunc::HashToField128Security => ObjectType::NativeField.max_size(),
                    _ => todo!("max value must be implemented for opcode {} ", op),
                }
            }
            Opcode::ToBits | Opcode::ToRadix => BigUint::zero(), //pointers do not overflow
        }
    }

    //Returns the number of elements and their type, of the output result corresponding to the OPCODE function.
    pub fn get_result_type(&self) -> (u32, ObjectType) {
        match self {
            Opcode::LowLevel(op) => {
                match op {
                    BlackBoxFunc::AES => (0, ObjectType::NotAnObject), //Not implemented
                    BlackBoxFunc::SHA256 => (32, ObjectType::Unsigned(8)),
                    BlackBoxFunc::Blake2s => (32, ObjectType::Unsigned(8)),
                    BlackBoxFunc::HashToField128Security => (1, ObjectType::NativeField),
                    BlackBoxFunc::MerkleMembership => (1, ObjectType::NativeField), //or bool?
                    BlackBoxFunc::SchnorrVerify => (1, ObjectType::NativeField),    //or bool?
                    BlackBoxFunc::Pedersen => (2, ObjectType::NativeField),
                    BlackBoxFunc::EcdsaSecp256k1 => (1, ObjectType::NativeField), //field?
                    BlackBoxFunc::FixedBaseScalarMul => (2, ObjectType::NativeField),
                    BlackBoxFunc::AND => (1, ObjectType::NativeField),
                    BlackBoxFunc::XOR => (1, ObjectType::NativeField),
                    BlackBoxFunc::RANGE => (0, ObjectType::NotAnObject),
                }
            }
            Opcode::ToBits => (FieldElement::max_num_bits(), ObjectType::Boolean),
            Opcode::ToRadix => (FieldElement::max_num_bits(), ObjectType::NativeField),
        }
    }
}
