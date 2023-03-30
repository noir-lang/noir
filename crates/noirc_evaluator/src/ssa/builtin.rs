use crate::ssa::{
    context::SsaContext,
    node::{NodeId, ObjectType},
};
use acvm::{acir::BlackBoxFunc, FieldElement};
use num_bigint::BigUint;
use num_traits::{One, Zero};

/// Opcode here refers to either a black box function
/// defined in ACIR, or a function which has its
/// function signature defined in Noir, but its
/// function definition is implemented in the compiler.
#[derive(Clone, Debug, Hash, Copy, PartialEq, Eq)]
pub(crate) enum Opcode {
    LowLevel(BlackBoxFunc),
    ToBits(Endian),
    ToRadix(Endian),
    Println(PrintlnInfo),
    Sort,

    Rand,
    Get2Notes,
    GetNNotes,
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
    pub(crate) fn lookup(op_name: &str) -> Option<Opcode> {
        match op_name {
            "to_le_bits" => Some(Opcode::ToBits(Endian::Little)),
            "to_be_bits" => Some(Opcode::ToBits(Endian::Big)),
            "to_le_radix" => Some(Opcode::ToRadix(Endian::Little)),
            "to_be_radix" => Some(Opcode::ToRadix(Endian::Big)),
            "rand" => Some(Opcode::Rand),
            "get_2_notes" => Some(Opcode::Get2Notes),
            "get_n_notes" => Some(Opcode::GetNNotes),
            "println" => {
                Some(Opcode::Println(PrintlnInfo { is_string_output: false, show_output: true }))
            }
            "arraysort" => Some(Opcode::Sort),
            _ => BlackBoxFunc::lookup(op_name).map(Opcode::LowLevel),
        }
    }

    fn name(&self) -> &str {
        match self {
            Opcode::LowLevel(op) => op.name(),
            Opcode::ToBits(endianness) => {
                if *endianness == Endian::Little {
                    "to_le_bits"
                } else {
                    "to_be_bits"
                }
            }
            Opcode::ToRadix(endianness) => {
                if *endianness == Endian::Little {
                    "to_le_radix"
                } else {
                    "to_be_radix"
                }
            }
            Opcode::Println(_) => "println",
            Opcode::Sort => "arraysort",
            Opcode::Rand => "rand",
            Opcode::Get2Notes => "get_2_notes",
            Opcode::GetNNotes => "get_n_notes",
        }
    }

    pub(crate) fn get_max_value(&self) -> BigUint {
        match self {
            Opcode::LowLevel(op) => {
                match op {
                    // Pointers do not overflow
                    BlackBoxFunc::SHA256
                    | BlackBoxFunc::Blake2s
                    | BlackBoxFunc::Pedersen
                    | BlackBoxFunc::FixedBaseScalarMul => BigUint::zero(),
                    // Verify returns zero or one
                    BlackBoxFunc::SchnorrVerify
                    | BlackBoxFunc::EcdsaSecp256k1
                    | BlackBoxFunc::MerkleMembership => BigUint::one(),
                    BlackBoxFunc::HashToField128Security => ObjectType::NativeField.max_size(),
                    BlackBoxFunc::AES => {
                        todo!("ICE: AES is unimplemented")
                    }
                    BlackBoxFunc::Keccak256 => {
                        todo!("ICE: Keccak256 is unimplemented")
                    }
                    BlackBoxFunc::RANGE | BlackBoxFunc::AND | BlackBoxFunc::XOR => {
                        unimplemented!("ICE: these opcodes do not have Noir builtin functions")
                    }
                }
            }
            Opcode::Rand => ObjectType::NativeField.max_size(),
            Opcode::Get2Notes
            | Opcode::GetNNotes
            | Opcode::ToBits(_)
            | Opcode::ToRadix(_)
            | Opcode::Println(_)
            | Opcode::Sort => BigUint::zero(), //pointers do not overflow
        }
    }

    /// Returns the number of elements that the `Opcode` should return
    /// and the type.
    pub(crate) fn get_result_type(&self, args: &[NodeId], ctx: &SsaContext) -> (u32, ObjectType) {
        match self {
            Opcode::LowLevel(op) => {
                match op {
                    BlackBoxFunc::AES => todo!("ICE: AES is unimplemented"),
                    BlackBoxFunc::Keccak256 => {
                        todo!("ICE: Keccak256 is unimplemented")
                    }
                    BlackBoxFunc::SHA256 | BlackBoxFunc::Blake2s => (32, ObjectType::Unsigned(8)),
                    BlackBoxFunc::HashToField128Security => (1, ObjectType::NativeField),
                    // See issue #775 on changing this to return a boolean
                    BlackBoxFunc::MerkleMembership
                    | BlackBoxFunc::SchnorrVerify
                    | BlackBoxFunc::EcdsaSecp256k1 => (1, ObjectType::NativeField),
                    BlackBoxFunc::Pedersen => (2, ObjectType::NativeField),
                    BlackBoxFunc::FixedBaseScalarMul => (2, ObjectType::NativeField),
                    BlackBoxFunc::RANGE | BlackBoxFunc::AND | BlackBoxFunc::XOR => {
                        unreachable!("ICE: these opcodes do not have Noir builtin functions")
                    }
                }
            }
            Opcode::ToBits(_) => (FieldElement::max_num_bits(), ObjectType::Boolean),
            Opcode::ToRadix(_) => (FieldElement::max_num_bits(), ObjectType::NativeField),
            Opcode::Println(_) => (0, ObjectType::NotAnObject),
            Opcode::Sort => {
                let a = super::mem::Memory::deref(ctx, args[0]).unwrap();
                (ctx.mem[a].len, ctx.mem[a].element_type)
            }
            Opcode::Rand => (1, ObjectType::NativeField),
            Opcode::Get2Notes => (26, ObjectType::NativeField),
            Opcode::GetNNotes => (13 * 1024, ObjectType::NativeField),
        }
    }
}

#[derive(Clone, Debug, Hash, Copy, PartialEq, Eq)]
pub(crate) struct PrintlnInfo {
    // We store strings as arrays and there is no differentiation between them in the SSA.
    // This bool simply states whether an array that is to be printed should be outputted as a utf8 string
    pub(crate) is_string_output: bool,
    // This is a flag used during `nargo test` to determine whether to display println output.
    pub(crate) show_output: bool,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub(crate) enum Endian {
    Big,
    Little,
}
