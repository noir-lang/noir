comptime global BN254_MODULUS_BE_BYTES: [u8] = &[
    48, 100, 78, 114, 225, 49, 160, 41, 184, 80, 69, 182, 129, 129, 88, 93, 40, 51, 232, 72, 121,
    185, 112, 145, 67, 225, 245, 147, 240, 0, 0, 1,
];

pub fn is_bn254() -> bool {
    comptime {
        // We can't use the `Eq` trait here due to limitations on calling non-comptime functions
        // defined within the same crate.
        let mut eq = true;

        let modulus_be_bytes = crate::field::modulus_be_bytes();
        // We can't do `BN254_MODULUS_BE_BYTES.len()` due to limitations on calling non-comptime functions.
        assert_eq(crate::field::modulus_num_bits(), 254);
        for i in 0..32 {
            eq &= modulus_be_bytes[i] == BN254_MODULUS_BE_BYTES[i];
        }

        eq
    }
}

mod test {
    use crate::compat::is_bn254;
    use super::BN254_MODULUS_BE_BYTES;

    #[test]
    fn is_bn254_returns_true_for_bn254_curve() {
        let result = is_bn254();
        assert_eq(result, true, "is_bn254() should return true for BN254 curve");
    }

    #[test]
    fn BN254_MODULUS_BE_BYTES_are_correct() {
        let expected: [u8] = [
            48, 100, 78, 114, 225, 49, 160, 41, 184, 80, 69, 182, 129, 129, 88, 93, 40, 51, 232, 72,
            121, 185, 112, 145, 67, 225, 245, 147, 240, 0, 0, 1,
        ];
        assert_eq(BN254_MODULUS_BE_BYTES, expected);
    }
}
