global BN254_MODULUS_BE_BYTES: [u8; 32] = [
    48, 100, 78, 114, 225, 49, 160, 41, 184, 80, 69, 182, 129, 129, 88, 93, 40, 51, 232, 72, 121, 185, 112, 145, 67, 225, 245, 147, 240, 0, 0, 1
];

pub fn is_bn254() -> bool {
    // TODO: refactor this once https://github.com/noir-lang/noir/issues/5245 is resolved.
    let modulus_bytes = crate::field::modulus_be_bytes();
    if modulus_bytes.len() == 32 {
        let mut modulus_bytes_array: [u8; 32] = [0; 32];
        for i in 0..32 {
            modulus_bytes_array[i] = modulus_bytes[i];
        }

        modulus_bytes_array == BN254_MODULUS_BE_BYTES
    } else {
        false
    }
}
