global BN254_MODULUS_BE_BYTES: [u8] = &[
    48, 100, 78, 114, 225, 49, 160, 41, 184, 80, 69, 182, 129, 129, 88, 93, 40, 51, 232, 72, 121, 185, 112, 145, 67, 225, 245, 147, 240, 0, 0, 1
];

pub fn is_bn254() -> bool {
    crate::field::modulus_be_bytes() == BN254_MODULUS_BE_BYTES
}
