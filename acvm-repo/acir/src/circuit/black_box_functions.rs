//! Black box functions are ACIR opcodes which rely on backends implementing support for specialized constraints.
//! This makes certain zk-snark unfriendly computations cheaper than if they were implemented in more basic constraints.

use serde::{Deserialize, Serialize};
#[cfg(test)]
use strum_macros::EnumIter;

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug, Hash, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(EnumIter))]
pub enum BlackBoxFunc {
    /// Bitwise AND.
    AND,
    /// Bitwise XOR.
    XOR,
    /// Range constraint to ensure that a [`FieldElement`][acir_field::FieldElement] can be represented in a specified number of bits.
    RANGE,
    /// Calculates the SHA256 hash of the inputs.
    SHA256,
    /// Calculates the Blake2s hash of the inputs.
    Blake2s,
    /// Calculates the Blake3 hash of the inputs.
    Blake3,
    /// Verifies a Schnorr signature over a curve which is "pairing friendly" with the curve on which the ACIR circuit is defined.
    ///
    /// The exact curve which this signature uses will vary based on the curve being used by ACIR.
    /// For example, the BN254 curve supports Schnorr signatures over the [Grumpkin][grumpkin] curve.
    ///
    /// [grumpkin]: https://hackmd.io/@aztec-network/ByzgNxBfd#2-Grumpkin---A-curve-on-top-of-BN-254-for-SNARK-efficient-group-operations
    SchnorrVerify,
    /// Calculates a Pedersen commitment to the inputs.
    PedersenCommitment,
    /// Calculates a Pedersen hash to the inputs.
    PedersenHash,
    /// Verifies a ECDSA signature over the secp256k1 curve.
    EcdsaSecp256k1,
    /// Verifies a ECDSA signature over the secp256r1 curve.
    EcdsaSecp256r1,
    /// Performs scalar multiplication over the embedded curve on which [`FieldElement`][acir_field::FieldElement] is defined.
    FixedBaseScalarMul,
    /// Calculates the Keccak256 hash of the inputs.
    Keccak256,
    /// Keccak Permutation function of 1600 width
    Keccakf1600,
    /// Compute a recursive aggregation object when verifying a proof inside another circuit.
    /// This outputted aggregation object will then be either checked in a top-level verifier or aggregated upon again.
    RecursiveAggregation,
}

impl std::fmt::Display for BlackBoxFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl BlackBoxFunc {
    pub fn name(&self) -> &'static str {
        match self {
            BlackBoxFunc::SHA256 => "sha256",
            BlackBoxFunc::SchnorrVerify => "schnorr_verify",
            BlackBoxFunc::Blake2s => "blake2s",
            BlackBoxFunc::Blake3 => "blake3",
            BlackBoxFunc::PedersenCommitment => "pedersen_commitment",
            BlackBoxFunc::PedersenHash => "pedersen_hash",
            BlackBoxFunc::EcdsaSecp256k1 => "ecdsa_secp256k1",
            BlackBoxFunc::FixedBaseScalarMul => "fixed_base_scalar_mul",
            BlackBoxFunc::AND => "and",
            BlackBoxFunc::XOR => "xor",
            BlackBoxFunc::RANGE => "range",
            BlackBoxFunc::Keccak256 => "keccak256",
            BlackBoxFunc::Keccakf1600 => "keccakf1600",
            BlackBoxFunc::RecursiveAggregation => "recursive_aggregation",
            BlackBoxFunc::EcdsaSecp256r1 => "ecdsa_secp256r1",
        }
    }
    pub fn lookup(op_name: &str) -> Option<BlackBoxFunc> {
        match op_name {
            "sha256" => Some(BlackBoxFunc::SHA256),
            "schnorr_verify" => Some(BlackBoxFunc::SchnorrVerify),
            "blake2s" => Some(BlackBoxFunc::Blake2s),
            "blake3" => Some(BlackBoxFunc::Blake3),
            "pedersen_commitment" => Some(BlackBoxFunc::PedersenCommitment),
            "pedersen_hash" => Some(BlackBoxFunc::PedersenHash),
            "ecdsa_secp256k1" => Some(BlackBoxFunc::EcdsaSecp256k1),
            "ecdsa_secp256r1" => Some(BlackBoxFunc::EcdsaSecp256r1),
            "fixed_base_scalar_mul" => Some(BlackBoxFunc::FixedBaseScalarMul),
            "and" => Some(BlackBoxFunc::AND),
            "xor" => Some(BlackBoxFunc::XOR),
            "range" => Some(BlackBoxFunc::RANGE),
            "keccak256" => Some(BlackBoxFunc::Keccak256),
            "keccakf1600" => Some(BlackBoxFunc::Keccakf1600),
            "recursive_aggregation" => Some(BlackBoxFunc::RecursiveAggregation),
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
