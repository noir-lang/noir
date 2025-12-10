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
    /// More details can be found at [crate::circuit::opcodes::BlackBoxFuncCall::AES128Encrypt]
    AES128Encrypt,
    /// More details can be found at [crate::circuit::opcodes::BlackBoxFuncCall::AND]
    AND,
    /// More details can be found at [crate::circuit::opcodes::BlackBoxFuncCall::XOR]
    XOR,
    /// More details can be found at [crate::circuit::opcodes::BlackBoxFuncCall::RANGE]
    RANGE,
    /// More details can be found at [crate::circuit::opcodes::BlackBoxFuncCall::Blake2s]
    Blake2s,
    /// More details can be found at [crate::circuit::opcodes::BlackBoxFuncCall::Blake3]
    Blake3,
    /// More details can be found at [crate::circuit::opcodes::BlackBoxFuncCall::EcdsaSecp256k1]
    EcdsaSecp256k1,
    /// More details can be found at [crate::circuit::opcodes::BlackBoxFuncCall::EcdsaSecp256r1]
    EcdsaSecp256r1,
    /// More details can be found at [crate::circuit::opcodes::BlackBoxFuncCall::MultiScalarMul]
    MultiScalarMul,
    /// More details can be found at [crate::circuit::opcodes::BlackBoxFuncCall::Keccakf1600]
    Keccakf1600,
    /// More details can be found at [crate::circuit::opcodes::BlackBoxFuncCall::RecursiveAggregation]
    RecursiveAggregation,
    /// More details can be found at [crate::circuit::opcodes::BlackBoxFuncCall::EmbeddedCurveAdd]
    EmbeddedCurveAdd,
    /// More details can be found at [crate::circuit::opcodes::BlackBoxFuncCall::Poseidon2Permutation]
    Poseidon2Permutation,
    /// More details can be found at [crate::circuit::opcodes::BlackBoxFuncCall::Sha256Compression]
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
            "poseidon2_permutation" => Some(BlackBoxFunc::Poseidon2Permutation),
            "sha256_compression" => Some(BlackBoxFunc::Sha256Compression),
            _ => None,
        }
    }

    pub fn has_side_effects(&self) -> bool {
        match self {
            BlackBoxFunc::RecursiveAggregation
            | BlackBoxFunc::MultiScalarMul
            | BlackBoxFunc::EmbeddedCurveAdd
            | BlackBoxFunc::EcdsaSecp256k1
            | BlackBoxFunc::EcdsaSecp256r1
            | BlackBoxFunc::RANGE => true,

            BlackBoxFunc::AES128Encrypt
            | BlackBoxFunc::AND
            | BlackBoxFunc::XOR
            | BlackBoxFunc::Blake2s
            | BlackBoxFunc::Blake3
            | BlackBoxFunc::Keccakf1600
            | BlackBoxFunc::Poseidon2Permutation
            | BlackBoxFunc::Sha256Compression => false,
        }
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
