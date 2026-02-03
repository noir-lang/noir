use acir::BlackBoxFunc;

use k256::{
    AffinePoint, EncodedPoint, ProjectivePoint, PublicKey,
    elliptic_curve::{
        PrimeField,
        ops::Reduce,
        scalar::IsHigh,
        sec1::{Coordinates, FromEncodedPoint, ToEncodedPoint},
    },
};
use k256::{Scalar, ecdsa::Signature};

use crate::BlackBoxResolutionError;

/// Verifies an ECDSA signature over the Secp256k1 elliptic curve.
///
/// This function implements ECDSA signature verification on the Secp256k1 curve
///
/// # Parameters:
///
/// * `hashed_msg` - The 32-byte hash of the message that was signed
/// * `public_key_x_bytes` - The x-coordinate of the public key (32 bytes, big-endian)
/// * `public_key_y_bytes` - The y-coordinate of the public key (32 bytes, big-endian)
/// * `signature` - The 64-byte signature in (r, s) format, where r and s are 32 bytes each
///
/// Returns `true` if the signature is valid, `false` otherwise.
///
/// The function returns an error if the following is not true:
/// - The signature components `r` and `s` must be non-zero
/// - The public key point must lie on the Secp256r1 curve
///
/// The function do not validate a signature if:
/// - The signature is not "low S" normalized per BIP 0062 to prevent malleability
///
/// The function will panic if `hashed_msg >= k256::Secp256k1::ORDER`.
/// According to ECDSA specification, the message hash leftmost bits should be truncated
/// up to the curve order length, and then reduced modulo the curve order.
pub(super) fn verify_signature(
    hashed_msg: &[u8; 32],
    public_key_x_bytes: &[u8; 32],
    public_key_y_bytes: &[u8; 32],
    signature: &[u8; 64],
) -> Result<bool, BlackBoxResolutionError> {
    // Convert the inputs into k256 data structures
    let Ok(signature) = Signature::try_from(signature.as_slice()) else {
        // Signature `r` and `s` are forbidden from being zero.
        return Err(BlackBoxResolutionError::Failed(
            BlackBoxFunc::EcdsaSecp256k1,
            "Signature provided for ECDSA verification is zero".to_string(),
        ));
    };

    let point = EncodedPoint::from_affine_coordinates(
        public_key_x_bytes.into(),
        public_key_y_bytes.into(),
        false,
    );

    let pubkey = PublicKey::from_encoded_point(&point);
    if pubkey.is_none().into() {
        // Public key must sit on the Secp256k1 curve.
        return Err(BlackBoxResolutionError::Failed(
            BlackBoxFunc::EcdsaSecp256k1,
            "Invalid public key provided for ECDSA verification".to_string(),
        ));
    }
    let pubkey = pubkey.unwrap();

    // Convert the hashed message to a scalar.
    // Per ECDSA specification (SEC 1, section 4.1.4), if `hashed_msg >= k256::Secp256k1::ORDER`,
    // the message hash should be reduced modulo the curve order.
    let z = <Scalar as Reduce<k256::U256>>::reduce(&k256::U256::from_be_slice(hashed_msg));

    // Finished converting bytes into data structures

    let r = signature.r();
    let s = signature.s();

    // Ensure signature is "low S" normalized ala BIP 0062
    if s.is_high().into() {
        log::warn!(
            "Signature provided for ECDSA verification is not properly normalized (high S value)"
        );
        return Ok(false);
    }

    let s_inv = s.invert().unwrap();
    let u1 = z * s_inv;
    let u2 = *r * s_inv;

    #[allow(non_snake_case)]
    let R: AffinePoint = ((ProjectivePoint::GENERATOR * u1)
        + (ProjectivePoint::from(*pubkey.as_affine()) * u2))
        .to_affine();

    // Compare R.x with signature's r component.
    match R.to_encoded_point(false).coordinates() {
        Coordinates::Uncompressed { x, y: _ } => {
            // The conversion from R.x to a scalar can fail if R.x >= curve_order (a possible but rare case).
            // In this case, the signature is invalid per ECDSA specification, so we return false.
            // The prover will handle this gracefully - it should generate a proof that fails verification.
            Ok(Scalar::from_repr(*x).into_option().map_or_else(
                || {
                    log::warn!(
                        "ECDSA Secp256k1 verification: R.x coordinate exceeds scalar field order - signature is invalid"
                    );
                    false
                },
                |scalar| scalar == *r,
            ))
        }
        Coordinates::Identity => Ok(false),
        _ => Err(BlackBoxResolutionError::Failed(
            BlackBoxFunc::EcdsaSecp256k1,
            "Unexpected coordinate encoding".to_string(),
        )),
    }
}

#[cfg(test)]
mod secp256k1_tests {
    use super::verify_signature;

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
        let valid = verify_signature(&HASHED_MESSAGE, &PUB_KEY_X, &PUB_KEY_Y, &SIGNATURE).unwrap();

        assert!(valid);
    }

    #[test]
    #[should_panic]
    fn rejects_signature_that_does_not_have_the_full_y_coordinate() {
        let mut pub_key_y_bytes = [0u8; 32];
        pub_key_y_bytes[31] = PUB_KEY_Y[31];

        verify_signature(&HASHED_MESSAGE, &PUB_KEY_X, &pub_key_y_bytes, &SIGNATURE).unwrap();
    }

    #[test]
    #[should_panic]
    fn rejects_invalid_signature() {
        // This signature is invalid as ECDSA specifies that `r` and `s` must be non-zero.
        let invalid_signature: [u8; 64] = [0x00; 64];

        verify_signature(&HASHED_MESSAGE, &PUB_KEY_X, &PUB_KEY_Y, &invalid_signature).unwrap();
    }

    #[test]
    #[should_panic]
    fn rejects_invalid_public_key() {
        let invalid_pub_key_x: [u8; 32] = [0xff; 32];
        let invalid_pub_key_y: [u8; 32] = [0xff; 32];

        verify_signature(&HASHED_MESSAGE, &invalid_pub_key_x, &invalid_pub_key_y, &SIGNATURE)
            .unwrap();
    }
}
