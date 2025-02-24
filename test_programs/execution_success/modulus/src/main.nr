fn main(bn254_modulus_be_bytes: [u8; 32], bn254_modulus_be_bits: [u1; 254]) {
    let modulus_size = std::field::modulus_num_bits();
    // NOTE: The constraints used in this circuit will only work when testing nargo with the plonk bn254 backend
    assert(modulus_size == 254);

    assert_reverse(
        std::field::modulus_be_bytes(),
        std::field::modulus_le_bytes(),
    );

    let modulus_be_byte_array = std::field::modulus_be_bytes();
    for i in 0..32 {
        assert(modulus_be_byte_array[i] == bn254_modulus_be_bytes[i]);
    }
    let modulus_le_byte_array = std::field::modulus_le_bytes();
    for i in 0..32 {
        assert(modulus_le_byte_array[i] == bn254_modulus_be_bytes[31 - i]);
    }

    let modulus_be_bits = std::field::modulus_be_bits();
    for i in 0..254 {
        assert(modulus_be_bits[i] == bn254_modulus_be_bits[i]);
    }
    let modulus_le_bits = std::field::modulus_le_bits();
    for i in 0..254 {
        assert(modulus_le_bits[i] == bn254_modulus_be_bits[253 - i]);
    }
}

fn assert_reverse(forwards: [u8], backwards: [u8]) {
    for i in 0..32 {
        assert_eq(forwards[i], backwards[31 - i]);
    }
}
