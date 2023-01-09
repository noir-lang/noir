use acvm::{acir::OPCODE, FieldElement};
use num_bigint::BigUint;
use num_traits::{One, Zero};

use super::node::ObjectType;

#[derive(Clone, Debug, Hash, Copy, PartialEq, Eq)]
pub enum Opcode {
    LowLevel(OPCODE),
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
            "to_bits" => Some(Opcode::ToBits),
            "to_radix" => Some(Opcode::ToRadix),
            _ => OPCODE::lookup(op_name).map(Opcode::LowLevel),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Opcode::LowLevel(op) => op.name(),
            Opcode::ToBits => "to_bits",
            Opcode::ToRadix => "to_radix",
        }
    }

    pub fn get_max_value(&self) -> BigUint {
        match self {
            Opcode::LowLevel(op) => {
                match op {
                    OPCODE::SHA256
                    | OPCODE::Blake2s
                    | OPCODE::Pedersen
                    | OPCODE::FixedBaseScalarMul => BigUint::zero(), //pointers do not overflow
                    OPCODE::SchnorrVerify | OPCODE::EcdsaSecp256k1 | OPCODE::MerkleMembership => {
                        BigUint::one()
                    } //verify returns 0 or 1
                    OPCODE::HashToField => ins.res_type.max_size(),
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
                    OPCODE::AES => (0, ObjectType::NotAnObject), //Not implemented
                    OPCODE::SHA256 => (32, ObjectType::Unsigned(8)),
                    OPCODE::Blake2s => (32, ObjectType::Unsigned(8)),
                    OPCODE::HashToField => (1, ObjectType::NativeField),
                    OPCODE::MerkleMembership => (1, ObjectType::NativeField), //or bool?
                    OPCODE::SchnorrVerify => (1, ObjectType::NativeField),    //or bool?
                    OPCODE::Pedersen => (2, ObjectType::NativeField),
                    OPCODE::EcdsaSecp256k1 => (1, ObjectType::NativeField), //field?
                    OPCODE::FixedBaseScalarMul => (2, ObjectType::NativeField),
                }
            }
            Opcode::ToBits => (FieldElement::max_num_bits(), ObjectType::Boolean),
            Opcode::ToRadix => (FieldElement::max_num_bits(), ObjectType::NativeField),
        }
    }
}
