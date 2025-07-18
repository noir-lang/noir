comptime global M31_MODULUS_BE_BYTES: [u8] = &[
    127,     255,     255,     255,
];

pub fn is_m31() -> bool {
    comptime {
        let mut eq = true;
        let modulus_be_bytes = crate::field::modulus_be_bytes();

        // modulus num bytes returns 31 as a u64 for m31
        assert_eq(crate::field::modulus_num_bits(), 31);

        for i in 0..4 {
            eq &= modulus_be_bytes[i] == M31_MODULUS_BE_BYTES[i];
        }

        eq
    }
}