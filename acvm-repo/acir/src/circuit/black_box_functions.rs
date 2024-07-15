//! Black box functions are ACIR opcodes which rely on backends implementing
//! support for specialized constraints.
//! This makes certain zk-snark unfriendly computations cheaper than if they were
//! implemented in more basic constraints.

use serde::{Deserialize, Serialize};
#[cfg(test)]
use strum_macros::EnumIter;

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug, Hash, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(EnumIter))]
pub enum BlackBoxFunc {
    /// Ciphers (encrypts) the provided plaintext using AES128 in CBC mode,
    /// padding the input using PKCS#7.
    /// - inputs: byte array `[u8; N]`
    /// - iv: initialization vector `[u8; 16]`
    /// - key: user key `[u8; 16]`
    /// - outputs: byte vector `[u8]` of length `input.len() + (16 - input.len() % 16)`
    AES128Encrypt,

    /// Performs the bitwise AND of `lhs` and `rhs`. `bit_size` must be the same for
    /// both inputs.
    /// - lhs: (witness, bit_size)
    /// - rhs: (witness, bit_size)
    /// - output: a witness whose value is constrained to be lhs AND rhs, as
    ///   bit_size bit integers
    AND,

    /// Performs the bitwise XOR of `lhs` and `rhs`. `bit_size` must be the same for
    /// both inputs.
    /// - lhs: (witness, bit_size)
    /// - rhs: (witness, bit_size)
    /// - output: a witness whose value is constrained to be lhs XOR rhs, as
    ///   bit_size bit integers
    XOR,

    /// Range constraint to ensure that a witness
    /// can be represented in the specified number of bits.
    /// - input: (witness, bit_size)
    RANGE,

    /// Computes SHA256 of the inputs
    /// - inputs are a byte array, i.e a vector of (witness, 8)
    /// - output is a byte array of len 32, i.e an array of 32 (witness, 8),
    ///   constrained to be the sha256 of the inputs.
    SHA256,

    /// Computes the Blake2s hash of the inputs, as specified in
    /// https://tools.ietf.org/html/rfc7693
    /// - inputs are a byte array, i.e a vector of (witness, 8)
    /// - output is a byte array of length 32, i.e. an array of 32
    /// (witness, 8), constrained to be the blake2s of the inputs.
    Blake2s,

    /// Computes the Blake3 hash of the inputs
    /// - inputs are a byte array, i.e a vector of (witness, 8)
    /// - output is a byte array of length 32, i.e an array of 32
    /// (witness, 8), constrained to be the blake3 of the inputs.
    Blake3,

    /// Verify a Schnorr signature over the embedded curve
    /// - inputs are:
    ///     - Public key as 2 (witness, 254)
    ///     - signature as a vector of 64 bytes (witness, 8)
    ///     - message as a vector of (witness, 8)
    /// - output: A witness representing the result of the signature
    /// verification; 0 for failure and 1 for success.
    ///
    /// Since the scalar field of the embedded curve is NOT the ACIR field, the
    /// `(r,s)` signature is represented as a 64 bytes array for the two field
    /// elements. On the other hand, the public key coordinates are ACIR fields.
    /// The proving system decides how the message is to be hashed. Barretenberg
    /// uses Blake2s.
    ///
    /// Verifies a Schnorr signature over a curve which is "pairing friendly"
    /// with the curve on which the ACIR circuit is defined.
    ///
    /// The exact curve which this signature uses will vary based on the curve
    /// being used by ACIR. For example, the BN254 curve supports Schnorr
    /// signatures over the [Grumpkin][grumpkin] curve.
    ///
    /// [grumpkin]: https://hackmd.io/@aztec-network/ByzgNxBfd#2-Grumpkin---A-curve-on-top-of-BN-254-for-SNARK-efficient-group-operations
    SchnorrVerify,
    /// Will be deprecated
    PedersenCommitment,
    /// Will be deprecated
    PedersenHash,
    /// Verifies a ECDSA signature over the secp256k1 curve.
    /// - inputs:
    ///     - x coordinate of public key as 32 bytes
    ///     - y coordinate of public key as 32 bytes
    ///     - the signature, as a 64 bytes array
    ///     - the hash of the message, as a vector of bytes
    /// - output: 0 for failure and 1 for success
    ///
    /// Inputs and outputs are similar to SchnorrVerify, except that because we
    /// use a different curve (secp256k1), the field elements involved in the
    /// signature and the public key are defined as an array of 32 bytes.
    /// Another difference is that we assume the message is already hashed.
    EcdsaSecp256k1,

    /// Verifies a ECDSA signature over the secp256r1 curve.
    ///
    /// Same as EcdsaSecp256k1, but done over another curve.
    EcdsaSecp256r1,

    /// Multiple scalar multiplication (MSM) with a variable base/input point
    /// (P) of the embedded curve. An MSM multiplies the points and scalars and
    /// sums the results.
    /// - input:
    ///     points (witness, N) a vector of x and y coordinates of input
    ///     points `[x1, y1, x2, y2,...]`.
    ///     scalars (witness, N) a vector of low and high limbs of input
    ///     scalars `[s1_low, s1_high, s2_low, s2_high, ...]`. (witness, N)
    ///     For Barretenberg, they must both be less than 128 bits.
    /// - output:
    ///     a tuple of `x` and `y` coordinates of output.
    ///     Points computed as `s_low*P+s_high*2^{128}*P`
    ///
    /// Because the Grumpkin scalar field is bigger than the ACIR field, we
    /// provide 2 ACIR fields representing the low and high parts of the Grumpkin
    /// scalar $a$: `a=low+high*2^{128}`, with `low, high < 2^{128}`
    MultiScalarMul,

    /// Computes the Keccak-256 (Ethereum version) of the inputs.
    /// - inputs: Vector of bytes (witness, 8)
    /// - outputs: Array of 32 bytes (witness, 8)
    Keccak256,

    /// Keccak Permutation function of width 1600
    /// - inputs: An array of 25 64-bit Keccak lanes that represent a keccak sponge of 1600 bits
    /// - outputs: The result of a keccak f1600 permutation on the input state. Also an array of 25 Keccak lanes.
    Keccakf1600,

    /// Compute a recursive aggregation object when verifying a proof inside
    /// another circuit.
    /// This outputted aggregation object will then be either checked in a
    /// top-level verifier or aggregated upon again.
    ///
    /// **Warning: this opcode is subject to change.**
    /// Note that the `254` in `(witness, 254)` refers to the upper bound of
    /// the `witness`.
    /// - verification_key: Vector of (witness, 254) representing the
    ///   verification key of the circuit being verified
    /// - public_inputs: Vector of (witness, 254)  representing the public
    ///   inputs corresponding to the proof being verified
    /// - key_hash: one (witness, 254). It should be the hash of the
    ///   verification key. Barretenberg expects the Pedersen hash of the
    ///   verification key
    ///
    /// Another thing that it does is preparing the verification of the proof.
    /// In order to fully verify a proof, some operations may still be required
    /// to be done by the final verifier. This is why this black box function
    /// does not say if verification is passing or not.
    ///
    /// This black box function does not fully verify a proof, what it does is
    /// verifying that the key_hash is indeed a hash of verification_key,
    /// allowing the user to use the verification key as private inputs and only
    /// have the key_hash as public input, which is more performant.
    ///
    /// If one of the recursive proofs you verify with the black box function does not
    /// verify, then the verification of the proof of the main ACIR program will
    /// ultimately fail.
    RecursiveAggregation,

    /// Addition over the embedded curve on which the witness is defined.
    EmbeddedCurveAdd,

    /// BigInt addition
    BigIntAdd,

    /// BigInt subtraction
    BigIntSub,

    /// BigInt multiplication
    BigIntMul,

    /// BigInt division
    BigIntDiv,

    /// BigInt from le bytes
    BigIntFromLeBytes,

    /// BigInt to le bytes
    BigIntToLeBytes,

    /// Permutation function of Poseidon2
    Poseidon2Permutation,

    /// SHA256 compression function
    /// - input: [(witness, 32); 16]
    /// - state: [(witness, 32); 8]
    /// - output: [(witness, 32); 8]
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
            BlackBoxFunc::SHA256 => "sha256",
            BlackBoxFunc::SchnorrVerify => "schnorr_verify",
            BlackBoxFunc::Blake2s => "blake2s",
            BlackBoxFunc::Blake3 => "blake3",
            BlackBoxFunc::EcdsaSecp256k1 => "ecdsa_secp256k1",
            BlackBoxFunc::MultiScalarMul => "multi_scalar_mul",
            BlackBoxFunc::EmbeddedCurveAdd => "embedded_curve_add",
            BlackBoxFunc::AND => "and",
            BlackBoxFunc::XOR => "xor",
            BlackBoxFunc::RANGE => "range",
            BlackBoxFunc::Keccak256 => "keccak256",
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
            BlackBoxFunc::PedersenCommitment => "pedersen_commitment",
            BlackBoxFunc::PedersenHash => "pedersen_hash",
        }
    }

    pub fn lookup(op_name: &str) -> Option<BlackBoxFunc> {
        match op_name {
            "aes128_encrypt" => Some(BlackBoxFunc::AES128Encrypt),
            "sha256" => Some(BlackBoxFunc::SHA256),
            "schnorr_verify" => Some(BlackBoxFunc::SchnorrVerify),
            "blake2s" => Some(BlackBoxFunc::Blake2s),
            "blake3" => Some(BlackBoxFunc::Blake3),
            "ecdsa_secp256k1" => Some(BlackBoxFunc::EcdsaSecp256k1),
            "ecdsa_secp256r1" => Some(BlackBoxFunc::EcdsaSecp256r1),
            "multi_scalar_mul" => Some(BlackBoxFunc::MultiScalarMul),
            "embedded_curve_add" => Some(BlackBoxFunc::EmbeddedCurveAdd),
            "and" => Some(BlackBoxFunc::AND),
            "xor" => Some(BlackBoxFunc::XOR),
            "range" => Some(BlackBoxFunc::RANGE),
            "keccak256" => Some(BlackBoxFunc::Keccak256),
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
            "pedersen_commitment" => Some(BlackBoxFunc::PedersenCommitment),
            "pedersen_hash" => Some(BlackBoxFunc::PedersenHash),
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
