comptime global M31_MODULUS_BE_BYTES: [u8] = &[
    127,     255,     255,     255,
]; // in big-endian

pub fn is_m31() -> bool {
    comptime {
        let mut eq = true;
        let modulus_be_bytes = crate::field::modulus_be_bytes();

        // modulus num bytes returns 31 as a u64 for m31
        assert_eq(crate::field::modulus_num_bits(), 31 as u64);

        for i in 0..4 {
            eq &= modulus_be_bytes[i] == M31_MODULUS_BE_BYTES[i];
        }

        eq
    }
}

// Not useful : will obv fail as field::modulus_be_bytes() will give 4 limbs

// comptime global BN254_MODULUS_BE_BYTES: [u8] = &[
//     48, 100, 78, 114, 225, 49, 160, 41, 184, 80, 69, 182, 129, 129, 88, 93, 40, 51, 232, 72, 121,
//     185, 112, 145, 67, 225, 245, 147, 240, 0, 0, 1,
// ];

// pub fn is_bn254() -> bool {
//     comptime {
//         // We can't use the `Eq` trait here due to limitations on calling non-comptime functions
//         // defined within the same crate.
//         let mut eq = true;

//         let modulus_be_bytes = crate::field::modulus_be_bytes();
//         // We can't do `BN254_MODULUS_BE_BYTES.len()` due to limitations on calling non-comptime functions.
//         assert_eq(crate::field::modulus_num_bits(), 254);
//         for i in 0..32 {
//             eq &= modulus_be_bytes[i] == BN254_MODULUS_BE_BYTES[i];
//         }

//         eq
//     }
// }