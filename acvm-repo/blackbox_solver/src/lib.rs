#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

//! This crate provides the implementation of BlackBox functions of ACIR and Brillig.
//! For functions that are backend-dependent, it provides a Trait [BlackBoxFunctionSolver] that must be implemented by the backend.
//! For functions that have a reference implementation, such as [keccak256], this crate exports the reference implementation directly.

use acir::BlackBoxFunc;
use blake2::digest::generic_array::GenericArray;
use blake2::{Blake2s256, Digest};
use sha2::Sha256;
use sha3::Keccak256;
use thiserror::Error;

mod curve_specific_solver;

pub use curve_specific_solver::{BlackBoxFunctionSolver, StubbedBlackBoxSolver};

#[derive(Clone, PartialEq, Eq, Debug, Error)]
pub enum BlackBoxResolutionError {
    #[error("failed to solve blackbox function: {0}, reason: {1}")]
    Failed(BlackBoxFunc, String),
}

pub fn sha256(inputs: &[u8]) -> Result<[u8; 32], BlackBoxResolutionError> {
    generic_hash_256::<Sha256>(inputs)
        .map_err(|err| BlackBoxResolutionError::Failed(BlackBoxFunc::SHA256, err))
}

pub fn blake2s(inputs: &[u8]) -> Result<[u8; 32], BlackBoxResolutionError> {
    generic_hash_256::<Blake2s256>(inputs)
        .map_err(|err| BlackBoxResolutionError::Failed(BlackBoxFunc::Blake2s, err))
}

pub fn blake3(inputs: &[u8]) -> Result<[u8; 32], BlackBoxResolutionError> {
    Ok(blake3::hash(inputs).into())
}

pub fn keccak256(inputs: &[u8]) -> Result<[u8; 32], BlackBoxResolutionError> {
    generic_hash_256::<Keccak256>(inputs)
        .map_err(|err| BlackBoxResolutionError::Failed(BlackBoxFunc::Keccak256, err))
}

const KECCAK_LANES: usize = 25;

pub fn keccakf1600(
    mut state: [u64; KECCAK_LANES],
) -> Result<[u64; KECCAK_LANES], BlackBoxResolutionError> {
    keccak::f1600(&mut state);
    Ok(state)
}

pub fn ecdsa_secp256k1_verify(
    hashed_msg: &[u8],
    public_key_x: &[u8; 32],
    public_key_y: &[u8; 32],
    signature: &[u8; 64],
) -> Result<bool, BlackBoxResolutionError> {
    Ok(verify_secp256k1_ecdsa_signature(hashed_msg, public_key_x, public_key_y, signature))
}

pub fn ecdsa_secp256r1_verify(
    hashed_msg: &[u8],
    public_key_x: &[u8; 32],
    public_key_y: &[u8; 32],
    signature: &[u8; 64],
) -> Result<bool, BlackBoxResolutionError> {
    Ok(verify_secp256r1_ecdsa_signature(hashed_msg, public_key_x, public_key_y, signature))
}

/// Does a generic hash of the inputs returning the resulting 32 bytes separately.
fn generic_hash_256<D: Digest>(message: &[u8]) -> Result<[u8; 32], String> {
    let output_bytes: [u8; 32] =
        D::digest(message).as_slice().try_into().map_err(|_| "digest should be 256 bits")?;

    Ok(output_bytes)
}

fn verify_secp256k1_ecdsa_signature(
    hashed_msg: &[u8],
    public_key_x_bytes: &[u8; 32],
    public_key_y_bytes: &[u8; 32],
    signature: &[u8; 64],
) -> bool {
    use k256::elliptic_curve::sec1::FromEncodedPoint;
    use k256::elliptic_curve::PrimeField;

    use k256::{ecdsa::Signature, Scalar};
    use k256::{
        elliptic_curve::{
            sec1::{Coordinates, ToEncodedPoint},
            IsHigh,
        },
        AffinePoint, EncodedPoint, ProjectivePoint, PublicKey,
    };
    // Convert the inputs into k256 data structures

    let Ok(signature) = Signature::try_from(signature.as_slice()) else {
        // Signature `r` and `s` are forbidden from being zero.
        return false;
    };

    let point = EncodedPoint::from_affine_coordinates(
        public_key_x_bytes.into(),
        public_key_y_bytes.into(),
        true,
    );

    let pubkey = PublicKey::from_encoded_point(&point);
    let pubkey = if pubkey.is_some().into() {
        pubkey.unwrap()
    } else {
        // Public key must sit on the Secp256k1 curve.
        return false;
    };

    // Note: This is incorrect as it will panic if `hashed_msg >= k256::Secp256k1::ORDER`.
    // In this scenario we should just take the leftmost bits from `hashed_msg` up to the group order length.
    let z = Scalar::from_repr(*GenericArray::from_slice(hashed_msg)).unwrap();

    // Finished converting bytes into data structures

    let r = signature.r();
    let s = signature.s();

    // Ensure signature is "low S" normalized ala BIP 0062
    if s.is_high().into() {
        return false;
    }

    let s_inv = s.invert().unwrap();
    let u1 = z * s_inv;
    let u2 = *r * s_inv;

    #[allow(non_snake_case)]
    let R: AffinePoint = ((ProjectivePoint::GENERATOR * u1)
        + (ProjectivePoint::from(*pubkey.as_affine()) * u2))
        .to_affine();

    match R.to_encoded_point(false).coordinates() {
        Coordinates::Uncompressed { x, y: _ } => Scalar::from_repr(*x).unwrap().eq(&r),
        _ => unreachable!("Point is uncompressed"),
    }
}

fn verify_secp256r1_ecdsa_signature(
    hashed_msg: &[u8],
    public_key_x_bytes: &[u8; 32],
    public_key_y_bytes: &[u8; 32],
    signature: &[u8; 64],
) -> bool {
    use p256::elliptic_curve::sec1::FromEncodedPoint;
    use p256::elliptic_curve::PrimeField;

    use p256::{ecdsa::Signature, Scalar};
    use p256::{
        elliptic_curve::{
            sec1::{Coordinates, ToEncodedPoint},
            IsHigh,
        },
        AffinePoint, EncodedPoint, ProjectivePoint, PublicKey,
    };

    // Convert the inputs into k256 data structures

    let Ok(signature) = Signature::try_from(signature.as_slice()) else {
        // Signature `r` and `s` are forbidden from being zero.
        return false;
    };

    let point = EncodedPoint::from_affine_coordinates(
        public_key_x_bytes.into(),
        public_key_y_bytes.into(),
        true,
    );

    let pubkey = PublicKey::from_encoded_point(&point);
    let pubkey = if pubkey.is_some().into() {
        pubkey.unwrap()
    } else {
        // Public key must sit on the Secp256r1 curve.
        return false;
    };

    // Note: This is incorrect as it will panic if `hashed_msg >= p256::NistP256::ORDER`.
    // In this scenario we should just take the leftmost bits from `hashed_msg` up to the group order length.
    let z = Scalar::from_repr(*GenericArray::from_slice(hashed_msg)).unwrap();

    // Finished converting bytes into data structures

    let r = signature.r();
    let s = signature.s();

    // Ensure signature is "low S" normalized ala BIP 0062
    if s.is_high().into() {
        return false;
    }

    let s_inv = s.invert().unwrap();
    let u1 = z * s_inv;
    let u2 = *r * s_inv;

    #[allow(non_snake_case)]
    let R: AffinePoint = ((ProjectivePoint::GENERATOR * u1)
        + (ProjectivePoint::from(*pubkey.as_affine()) * u2))
        .to_affine();

    match R.to_encoded_point(false).coordinates() {
        Coordinates::Uncompressed { x, y: _ } => Scalar::from_repr(*x).unwrap().eq(&r),
        _ => unreachable!("Point is uncompressed"),
    }
}

#[cfg(test)]
mod keccakf1600_tests {
    use crate::keccakf1600;

    #[test]
    fn sanity_check() {
        // Test vectors are copied from XKCP (eXtended Keccak Code Package)
        // https://github.com/XKCP/XKCP/blob/master/tests/TestVectors/KeccakF-1600-IntermediateValues.txt
        let zero_state = [0u64; 25];

        let expected_state_first = [
            0xF1258F7940E1DDE7,
            0x84D5CCF933C0478A,
            0xD598261EA65AA9EE,
            0xBD1547306F80494D,
            0x8B284E056253D057,
            0xFF97A42D7F8E6FD4,
            0x90FEE5A0A44647C4,
            0x8C5BDA0CD6192E76,
            0xAD30A6F71B19059C,
            0x30935AB7D08FFC64,
            0xEB5AA93F2317D635,
            0xA9A6E6260D712103,
            0x81A57C16DBCF555F,
            0x43B831CD0347C826,
            0x01F22F1A11A5569F,
            0x05E5635A21D9AE61,
            0x64BEFEF28CC970F2,
            0x613670957BC46611,
            0xB87C5A554FD00ECB,
            0x8C3EE88A1CCF32C8,
            0x940C7922AE3A2614,
            0x1841F924A2C509E4,
            0x16F53526E70465C2,
            0x75F644E97F30A13B,
            0xEAF1FF7B5CECA249,
        ];
        let expected_state_second = [
            0x2D5C954DF96ECB3C,
            0x6A332CD07057B56D,
            0x093D8D1270D76B6C,
            0x8A20D9B25569D094,
            0x4F9C4F99E5E7F156,
            0xF957B9A2DA65FB38,
            0x85773DAE1275AF0D,
            0xFAF4F247C3D810F7,
            0x1F1B9EE6F79A8759,
            0xE4FECC0FEE98B425,
            0x68CE61B6B9CE68A1,
            0xDEEA66C4BA8F974F,
            0x33C43D836EAFB1F5,
            0xE00654042719DBD9,
            0x7CF8A9F009831265,
            0xFD5449A6BF174743,
            0x97DDAD33D8994B40,
            0x48EAD5FC5D0BE774,
            0xE3B8C8EE55B7B03C,
            0x91A0226E649E42E9,
            0x900E3129E7BADD7B,
            0x202A9EC5FAA3CCE8,
            0x5B3402464E1C3DB6,
            0x609F4E62A44C1059,
            0x20D06CD26A8FBF5C,
        ];

        let state_first = keccakf1600(zero_state).unwrap();
        let state_second = keccakf1600(state_first).unwrap();

        assert_eq!(state_first, expected_state_first);
        assert_eq!(state_second, expected_state_second);
    }
}

#[cfg(test)]
mod secp256k1_tests {
    use super::verify_secp256k1_ecdsa_signature;

    // 0x3a73f4123a5cd2121f21cd7e8d358835476949d035d9c2da6806b4633ac8c1e2,
    const HASHED_MESSAGE: [u8; 32] = [
        0x3a, 0x73, 0xf4, 0x12, 0x3a, 0x5c, 0xd2, 0x12, 0x1f, 0x21, 0xcd, 0x7e, 0x8d, 0x35, 0x88,
        0x35, 0x47, 0x69, 0x49, 0xd0, 0x35, 0xd9, 0xc2, 0xda, 0x68, 0x06, 0xb4, 0x63, 0x3a, 0xc8,
        0xc1, 0xe2,
    ];
    // 0xa0434d9e47f3c86235477c7b1ae6ae5d3442d49b1943c2b752a68e2a47e247c7
    const PUB_KEY_X: [u8; 32] = [
        0xa0, 0x43, 0x4d, 0x9e, 0x47, 0xf3, 0xc8, 0x62, 0x35, 0x47, 0x7c, 0x7b, 0x1a, 0xe6, 0xae,
        0x5d, 0x34, 0x42, 0xd4, 0x9b, 0x19, 0x43, 0xc2, 0xb7, 0x52, 0xa6, 0x8e, 0x2a, 0x47, 0xe2,
        0x47, 0xc7,
    ];
    // 0x893aba425419bc27a3b6c7e693a24c696f794c2ed877a1593cbee53b037368d7
    const PUB_KEY_Y: [u8; 32] = [
        0x89, 0x3a, 0xba, 0x42, 0x54, 0x19, 0xbc, 0x27, 0xa3, 0xb6, 0xc7, 0xe6, 0x93, 0xa2, 0x4c,
        0x69, 0x6f, 0x79, 0x4c, 0x2e, 0xd8, 0x77, 0xa1, 0x59, 0x3c, 0xbe, 0xe5, 0x3b, 0x03, 0x73,
        0x68, 0xd7,
    ];
    // 0xe5081c80ab427dc370346f4a0e31aa2bad8d9798c38061db9ae55a4e8df454fd28119894344e71b78770cc931d61f480ecbb0b89d6eb69690161e49a715fcd55
    const SIGNATURE: [u8; 64] = [
        0xe5, 0x08, 0x1c, 0x80, 0xab, 0x42, 0x7d, 0xc3, 0x70, 0x34, 0x6f, 0x4a, 0x0e, 0x31, 0xaa,
        0x2b, 0xad, 0x8d, 0x97, 0x98, 0xc3, 0x80, 0x61, 0xdb, 0x9a, 0xe5, 0x5a, 0x4e, 0x8d, 0xf4,
        0x54, 0xfd, 0x28, 0x11, 0x98, 0x94, 0x34, 0x4e, 0x71, 0xb7, 0x87, 0x70, 0xcc, 0x93, 0x1d,
        0x61, 0xf4, 0x80, 0xec, 0xbb, 0x0b, 0x89, 0xd6, 0xeb, 0x69, 0x69, 0x01, 0x61, 0xe4, 0x9a,
        0x71, 0x5f, 0xcd, 0x55,
    ];

    #[test]
    fn verifies_valid_signature_with_low_s_value() {
        let valid =
            verify_secp256k1_ecdsa_signature(&HASHED_MESSAGE, &PUB_KEY_X, &PUB_KEY_Y, &SIGNATURE);

        assert!(valid);
    }

    #[test]
    fn rejects_invalid_signature() {
        // This signature is invalid as ECDSA specifies that `r` and `s` must be non-zero.
        let invalid_signature: [u8; 64] = [0x00; 64];

        let valid = verify_secp256k1_ecdsa_signature(
            &HASHED_MESSAGE,
            &PUB_KEY_X,
            &PUB_KEY_Y,
            &invalid_signature,
        );
        assert!(!valid);
    }

    #[test]
    fn rejects_invalid_public_key() {
        let invalid_pub_key_x: [u8; 32] = [0xff; 32];
        let invalid_pub_key_y: [u8; 32] = [0xff; 32];

        let valid = verify_secp256k1_ecdsa_signature(
            &HASHED_MESSAGE,
            &invalid_pub_key_x,
            &invalid_pub_key_y,
            &SIGNATURE,
        );

        assert!(!valid);
    }

    #[test]
    #[ignore = "ECDSA verification does not currently handle long hashes correctly"]
    fn trims_overly_long_hashes_to_correct_length() {
        let mut long_hashed_message = HASHED_MESSAGE.to_vec();
        long_hashed_message.push(0xff);

        let valid = verify_secp256k1_ecdsa_signature(
            &long_hashed_message,
            &PUB_KEY_X,
            &PUB_KEY_Y,
            &SIGNATURE,
        );

        assert!(valid);
    }
}

#[cfg(test)]
mod secp256r1_tests {
    use super::verify_secp256r1_ecdsa_signature;

    // 0x54705ba3baafdbdfba8c5f9a70f7a89bee98d906b53e31074da7baecdc0da9ad
    const HASHED_MESSAGE: [u8; 32] = [
        84, 112, 91, 163, 186, 175, 219, 223, 186, 140, 95, 154, 112, 247, 168, 155, 238, 152, 217,
        6, 181, 62, 49, 7, 77, 167, 186, 236, 220, 13, 169, 173,
    ];
    // 0x550f471003f3df97c3df506ac797f6721fb1a1fb7b8f6f83d224498a65c88e24
    const PUB_KEY_X: [u8; 32] = [
        85, 15, 71, 16, 3, 243, 223, 151, 195, 223, 80, 106, 199, 151, 246, 114, 31, 177, 161, 251,
        123, 143, 111, 131, 210, 36, 73, 138, 101, 200, 142, 36,
    ];
    // 0x136093d7012e509a73715cbd0b00a3cc0ff4b5c01b3ffa196ab1fb327036b8e6
    const PUB_KEY_Y: [u8; 32] = [
        19, 96, 147, 215, 1, 46, 80, 154, 115, 113, 92, 189, 11, 0, 163, 204, 15, 244, 181, 192,
        27, 63, 250, 25, 106, 177, 251, 50, 112, 54, 184, 230,
    ];
    // 0x2c70a8d084b62bfc5ce03641caf9f72ad4da8c81bfe6ec9487bb5e1bef62a13218ad9ee29eaf351fdc50f1520c425e9b908a07278b43b0ec7b872778c14e0784
    const SIGNATURE: [u8; 64] = [
        44, 112, 168, 208, 132, 182, 43, 252, 92, 224, 54, 65, 202, 249, 247, 42, 212, 218, 140,
        129, 191, 230, 236, 148, 135, 187, 94, 27, 239, 98, 161, 50, 24, 173, 158, 226, 158, 175,
        53, 31, 220, 80, 241, 82, 12, 66, 94, 155, 144, 138, 7, 39, 139, 67, 176, 236, 123, 135,
        39, 120, 193, 78, 7, 132,
    ];

    #[test]
    fn verifies_valid_signature_with_low_s_value() {
        let valid =
            verify_secp256r1_ecdsa_signature(&HASHED_MESSAGE, &PUB_KEY_X, &PUB_KEY_Y, &SIGNATURE);

        assert!(valid);
    }

    #[test]
    fn rejects_invalid_signature() {
        // This signature is invalid as ECDSA specifies that `r` and `s` must be non-zero.
        let invalid_signature: [u8; 64] = [0x00; 64];

        let valid = verify_secp256r1_ecdsa_signature(
            &HASHED_MESSAGE,
            &PUB_KEY_X,
            &PUB_KEY_Y,
            &invalid_signature,
        );
        assert!(!valid);
    }

    #[test]
    fn rejects_invalid_public_key() {
        let invalid_pub_key_x: [u8; 32] = [0xff; 32];
        let invalid_pub_key_y: [u8; 32] = [0xff; 32];

        let valid = verify_secp256r1_ecdsa_signature(
            &HASHED_MESSAGE,
            &invalid_pub_key_x,
            &invalid_pub_key_y,
            &SIGNATURE,
        );

        assert!(!valid);
    }

    #[test]
    #[ignore = "ECDSA verification does not currently handle long hashes correctly"]
    fn trims_overly_long_hashes_to_correct_length() {
        let mut long_hashed_message = HASHED_MESSAGE.to_vec();
        long_hashed_message.push(0xff);

        let valid = verify_secp256r1_ecdsa_signature(
            &long_hashed_message,
            &PUB_KEY_X,
            &PUB_KEY_Y,
            &SIGNATURE,
        );

        assert!(valid);
    }
}
