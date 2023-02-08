use crate::ssa::node::ObjectType;
use acvm::{acir::BlackBoxFunc, FieldElement};
use num_bigint::BigUint;
use num_traits::{One, Zero};

/// Opcode here refers to either a black box function
/// defined in ACIR, or a function which has its
/// function signature defined in Noir, but its
/// function definition is implemented in the compiler.
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
    /// Searches for an `Opcode` using its
    /// string equivalent name.
    ///
    /// Returns `None` if there is no string that
    /// corresponds to any of the opcodes.
    pub fn lookup(op_name: &str) -> Option<Opcode> {
        match op_name {
            "to_le_bits" => Some(Opcode::ToBits),
            "to_radix" => Some(Opcode::ToRadix),
            _ => BlackBoxFunc::lookup(op_name).map(Opcode::LowLevel),
        }
    }

    fn name(&self) -> &str {
        match self {
            Opcode::LowLevel(op) => op.name(),
            Opcode::ToBits => "to_le_bits",
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
                    _ => unreachable!("ICE: max value should be implemented for opcode {} ", op),
                }
            }
            Opcode::ToBits | Opcode::ToRadix => BigUint::zero(), //pointers do not overflow
        }
    }

    /// Returns the number of elements that the `Opcode` should return
    /// and the type.
    pub fn get_result_type(&self) -> (u32, ObjectType) {
        match self {
            Opcode::LowLevel(op) => {
                match op {
                    BlackBoxFunc::AES => todo!("ICE: AES is unimplemented"),
                    BlackBoxFunc::SHA256 => (32, ObjectType::Unsigned(8)),
                    BlackBoxFunc::Blake2s => (32, ObjectType::Unsigned(8)),
                    BlackBoxFunc::HashToField128Security => (1, ObjectType::NativeField),
                    // See issue #775 on changing this to return a boolean
                    BlackBoxFunc::MerkleMembership => (1, ObjectType::NativeField),
                    BlackBoxFunc::SchnorrVerify => (1, ObjectType::NativeField),
                    BlackBoxFunc::Pedersen => (2, ObjectType::NativeField),
                    BlackBoxFunc::EcdsaSecp256k1 => (1, ObjectType::NativeField),
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
