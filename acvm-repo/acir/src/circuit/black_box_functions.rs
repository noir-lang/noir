//! Black box functions are ACIR opcodes which rely on backends implementing
//! support for specialized constraints.
//! This makes certain zk-snark unfriendly computations cheaper than if they were
//! implemented in more basic constraints.

use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

/// Representation of available black box function names.
/// This enum should be used to represent a black box before we have set up the
/// appropriate inputs and outputs. At which point it should be converted to a [crate::circuit::opcodes::BlackBoxFuncCall]
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug, Hash, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIter)]
pub enum BlackBoxFunc {
    AES128Encrypt,
    AND,
    XOR,
    RANGE,
    Blake2s,
    Blake3,
    EcdsaSecp256k1,
    EcdsaSecp256r1,
    MultiScalarMul,
    Keccakf1600,
    RecursiveAggregation,
    EmbeddedCurveAdd,
    BigIntAdd,
    BigIntSub,
    BigIntMul,
    BigIntDiv,
    BigIntFromLeBytes,
    BigIntToLeBytes,
    Poseidon2Permutation,
    Sha256Compression,
}

impl std::fmt::Display for BlackBoxFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl BlackBoxFunc {
    pub fn name(&self) -> &'static str {
        match self {
            BlackBoxFunc::AES128Encrypt => "aes128_encrypt",
            BlackBoxFunc::Blake2s => "blake2s",
            BlackBoxFunc::Blake3 => "blake3",
            BlackBoxFunc::EcdsaSecp256k1 => "ecdsa_secp256k1",
            BlackBoxFunc::MultiScalarMul => "multi_scalar_mul",
            BlackBoxFunc::EmbeddedCurveAdd => "embedded_curve_add",
            BlackBoxFunc::AND => "and",
            BlackBoxFunc::XOR => "xor",
            BlackBoxFunc::RANGE => "range",
            BlackBoxFunc::Keccakf1600 => "keccakf1600",
            BlackBoxFunc::RecursiveAggregation => "recursive_aggregation",
            BlackBoxFunc::EcdsaSecp256r1 => "ecdsa_secp256r1",
            BlackBoxFunc::BigIntAdd => "bigint_add",
            BlackBoxFunc::BigIntSub => "bigint_sub",
            BlackBoxFunc::BigIntMul => "bigint_mul",
            BlackBoxFunc::BigIntDiv => "bigint_div",
            BlackBoxFunc::BigIntFromLeBytes => "bigint_from_le_bytes",
            BlackBoxFunc::BigIntToLeBytes => "bigint_to_le_bytes",
            BlackBoxFunc::Poseidon2Permutation => "poseidon2_permutation",
            BlackBoxFunc::Sha256Compression => "sha256_compression",
        }
    }

    pub fn lookup(op_name: &str) -> Option<BlackBoxFunc> {
        match op_name {
            "aes128_encrypt" => Some(BlackBoxFunc::AES128Encrypt),
            "blake2s" => Some(BlackBoxFunc::Blake2s),
            "blake3" => Some(BlackBoxFunc::Blake3),
            "ecdsa_secp256k1" => Some(BlackBoxFunc::EcdsaSecp256k1),
            "ecdsa_secp256r1" => Some(BlackBoxFunc::EcdsaSecp256r1),
            "multi_scalar_mul" => Some(BlackBoxFunc::MultiScalarMul),
            "embedded_curve_add" => Some(BlackBoxFunc::EmbeddedCurveAdd),
            "and" => Some(BlackBoxFunc::AND),
            "xor" => Some(BlackBoxFunc::XOR),
            "range" => Some(BlackBoxFunc::RANGE),
            "keccakf1600" => Some(BlackBoxFunc::Keccakf1600),
            "recursive_aggregation" => Some(BlackBoxFunc::RecursiveAggregation),
            "bigint_add" => Some(BlackBoxFunc::BigIntAdd),
            "bigint_sub" => Some(BlackBoxFunc::BigIntSub),
            "bigint_mul" => Some(BlackBoxFunc::BigIntMul),
            "bigint_div" => Some(BlackBoxFunc::BigIntDiv),
            "bigint_from_le_bytes" => Some(BlackBoxFunc::BigIntFromLeBytes),
            "bigint_to_le_bytes" => Some(BlackBoxFunc::BigIntToLeBytes),
            "poseidon2_permutation" => Some(BlackBoxFunc::Poseidon2Permutation),
            "sha256_compression" => Some(BlackBoxFunc::Sha256Compression),
            _ => None,
        }
    }

    pub fn is_valid_black_box_func_name(op_name: &str) -> bool {
        BlackBoxFunc::lookup(op_name).is_some()
    }
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use crate::BlackBoxFunc;

    #[test]
    fn consistent_function_names() {
        for bb_func in BlackBoxFunc::iter() {
            let resolved_func = BlackBoxFunc::lookup(bb_func.name()).unwrap_or_else(|| {
                panic!("BlackBoxFunc::lookup couldn't find black box function {bb_func}")
            });
            assert_eq!(
                resolved_func, bb_func,
                "BlackBoxFunc::lookup returns unexpected BlackBoxFunc"
            );
        }
    }
}
