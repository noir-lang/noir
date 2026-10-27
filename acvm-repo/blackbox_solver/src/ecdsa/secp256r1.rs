#[cfg(test)]
mod secp256r1_tests {
    use super::super::ecdsa_secp256r1_verify as verify_signature;
    use super::super::nonce_point_x;
    use k256::elliptic_curve::{Curve, bigint::U256};

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
    fn signature_does_not_verify_when_does_not_have_the_full_y_coordinate() {
        let mut pub_key_y_bytes = [0u8; 32];
        pub_key_y_bytes[31] = PUB_KEY_Y[31];
        let result =
            verify_signature(&HASHED_MESSAGE, &PUB_KEY_X, &pub_key_y_bytes, &SIGNATURE).unwrap();
        assert!(!result);
    }

    #[test]
    fn signature_does_not_verify_on_invalid_signature() {
        // This signature is invalid as ECDSA specifies that `r` and `s` must be non-zero.
        let invalid_signature: [u8; 64] = [0x00; 64];
        let result =
            verify_signature(&HASHED_MESSAGE, &PUB_KEY_X, &PUB_KEY_Y, &invalid_signature).unwrap();
        assert!(!result);
    }

    #[test]
    fn signature_does_not_verify_on_invalid_public_key() {
        let invalid_pub_key_x: [u8; 32] = [0xff; 32];
        let invalid_pub_key_y: [u8; 32] = [0xff; 32];
        let result =
            verify_signature(&HASHED_MESSAGE, &invalid_pub_key_x, &invalid_pub_key_y, &SIGNATURE)
                .unwrap();
        assert!(!result);
    }

    #[test]
    fn signature_does_not_verify_when_hashed_msg_exceeds_curve_order() {
        // All 0xFF bytes is larger than the secp256r1 curve order
        // (0xFFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632551).
        // The function should reduce it modulo the order, not panic.
        let oversized_hash: [u8; 32] = [0xff; 32];

        // The result will be false (signature doesn't match the reduced hash), but no panic.
        let result = verify_signature(&oversized_hash, &PUB_KEY_X, &PUB_KEY_Y, &SIGNATURE).unwrap();
        assert!(!result);
    }

    #[test]
    fn signature_does_not_verify_when_hashed_msg_equals_curve_order() {
        // The exact secp256r1 curve order.
        let curve_order: [u8; 32] = [
            0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xBC, 0xE6, 0xFA, 0xAD, 0xA7, 0x17, 0x9E, 0x84, 0xF3, 0xB9, 0xCA, 0xC2,
            0xFC, 0x63, 0x25, 0x51,
        ];

        let result = verify_signature(&curve_order, &PUB_KEY_X, &PUB_KEY_Y, &SIGNATURE).unwrap();
        assert!(!result);
    }

    #[test]
    fn verifies_signature_whose_nonce_point_x_coordinate_exceeds_curve_order() {
        // A signature where `R.x` lies in `[n, p)`: `R.x = n + 3`, so `r = R.x mod n = 3`.
        // Per SEC 1 section 4.1.4 the verifier compares `R.x mod n` against `r`, so this is valid.
        // 0xdeadbeef
        let hashed_msg: [u8; 32] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0xde, 0xad, 0xbe, 0xef,
        ];
        // 0x963472c465d877cef16f539910da1ea37ea00cde8f5c801aa382c0f3c4967a96
        let pub_key_x: [u8; 32] = [
            0x96, 0x34, 0x72, 0xc4, 0x65, 0xd8, 0x77, 0xce, 0xf1, 0x6f, 0x53, 0x99, 0x10, 0xda,
            0x1e, 0xa3, 0x7e, 0xa0, 0x0c, 0xde, 0x8f, 0x5c, 0x80, 0x1a, 0xa3, 0x82, 0xc0, 0xf3,
            0xc4, 0x96, 0x7a, 0x96,
        ];
        // 0x1ddb76709fd05c2d137b17493c90156e012673308e8fb12fdf7ff8bf638aa2ee
        let pub_key_y: [u8; 32] = [
            0x1d, 0xdb, 0x76, 0x70, 0x9f, 0xd0, 0x5c, 0x2d, 0x13, 0x7b, 0x17, 0x49, 0x3c, 0x90,
            0x15, 0x6e, 0x01, 0x26, 0x73, 0x30, 0x8e, 0x8f, 0xb1, 0x2f, 0xdf, 0x7f, 0xf8, 0xbf,
            0x63, 0x8a, 0xa2, 0xee,
        ];
        // r = 3, s = 0x3039
        let mut signature = [0u8; 64];
        signature[31] = 0x03;
        signature[62] = 0x30;
        signature[63] = 0x39;

        let order = p256::NistP256::ORDER.get();
        let nonce_point_x =
            nonce_point_x::<p256::NistP256>(&hashed_msg, &pub_key_x, &pub_key_y, &signature);
        assert_eq!(nonce_point_x, order.wrapping_add(&U256::from_u8(3)));

        let valid = verify_signature(&hashed_msg, &pub_key_x, &pub_key_y, &signature).unwrap();
        assert!(valid);

        // Any other `r` must still be rejected.
        signature[31] = 0x04;
        let valid = verify_signature(&hashed_msg, &pub_key_x, &pub_key_y, &signature).unwrap();
        assert!(!valid);
    }
}
