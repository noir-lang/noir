use acir::BlackBoxFunc;

use p256::{
    AffinePoint, EncodedPoint, ProjectivePoint, PublicKey,
    elliptic_curve::{
        PrimeField,
        ops::Reduce,
        scalar::IsHigh,
        sec1::{Coordinates, FromEncodedPoint, ToEncodedPoint},
    },
};
use p256::{Scalar, ecdsa::Signature};

use crate::BlackBoxResolutionError;

/// Verifies an ECDSA signature over the Secp256r1 elliptic curve.
///
/// This function implements ECDSA signature verification on the Secp256r1 curve
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
/// The function will panic if `hashed_msg >= p256::NistP256::ORDER`.
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
            BlackBoxFunc::EcdsaSecp256r1,
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
        // Public key must sit on the Secp256r1 curve.
        return Err(BlackBoxResolutionError::Failed(
            BlackBoxFunc::EcdsaSecp256k1,
            "Invalid public key provided for ECDSA verification".to_string(),
        ));
    };
    let pubkey = pubkey.unwrap();

    // Convert the hashed message to a scalar.
    // Per ECDSA specification (SEC 1, section 4.1.4), if `hashed_msg >= p256::NistP256::ORDER`,
    // the message hash should be reduced modulo the curve order.
    let z = <Scalar as Reduce<p256::U256>>::reduce(&p256::U256::from_be_slice(hashed_msg));

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

    match R.to_encoded_point(false).coordinates() {
        Coordinates::Uncompressed { x, y: _ } => {
            // The conversion from R.x to a scalar can fail if R.x >= curve_order (a possible but rare case).
            // In this case, the signature is invalid per ECDSA specification, so we return false.
            // The prover will handle this gracefully - it should generate a proof that fails verification.
            Ok(Scalar::from_repr(*x).into_option().map_or_else(
                || {
                    log::warn!("Failed to convert R.x coordinate to scalar for ECDSA verification");
                    false
                },
                |scalar| scalar == *r,
            ))
        }
        Coordinates::Identity => Ok(false),
        _ => Err(BlackBoxResolutionError::Failed(
            BlackBoxFunc::EcdsaSecp256r1,
            "Unexpected coordinate encoding".to_string(),
        )),
    }
}

#[cfg(test)]
mod secp256r1_tests {
    use super::verify_signature;

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
