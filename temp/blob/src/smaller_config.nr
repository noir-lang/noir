use dep::bigint::{BigNum, fields::bls12_381Fr::BLS12_381_Fr_Params};

type F = BigNum<3, BLS12_381_Fr_Params>;

global FIELDS_PER_BLOB: u32 = 8; // actually 4096
global LOG_FIELDS_PER_BLOB: u32 = 3; // actually 12

// The following three fields are more meaningful computation in the proper config.nr:
global EXTRA_FIELDS_PER_BLOB: u32 = 0; // 0 = floor(8 FIELDS_PER_BLOB / 254 noir_field_bits).
global NOIR_FIELDS_PER_BLOB = FIELDS_PER_BLOB + EXTRA_FIELDS_PER_BLOB; // It's a more meaningful computation in the proper config.nr.
global FIELDS_CARRYING_AN_EXTRA_BIT_PER_BLOB: u32 = 0; // EXTRA_FIELDS_PER_BLOB * 254 = 0. It's a more meaningful computation in the proper config.nr.

global D: F = BigNum { limbs: [8, 0, 0] };
global D_INV =
    BigNum { limbs: [0x45ef829ffe907f1fffffff20000001, 0xf268c469cd9f2cd29d07086d9d04a9, 0x656f] };

global ROOTS: [F; FIELDS_PER_BLOB] = [
    BigNum {
        limbs: [0x000000000000000000000000000001, 0x000000000000000000000000000000, 0x000000],
    },
    BigNum {
        limbs: [0xbda402fffe5bfeffffffff00000000, 0xa753299d7d483339d80809a1d80553, 0x0073ed],
    },
    BigNum {
        limbs: [0x030002760300000001000000000000, 0x0000000000008d51ccce760304d0ec, 0x000000],
    },
    BigNum {
        limbs: [0xbaa40089fb5bfefffeffff00000001, 0xa753299d7d47a5e80b39939ed33467, 0x0073ed],
    },
    BigNum {
        limbs: [0x8b21c28713b7007228fd3397743f7a, 0x66f603fa66e78c0625cd70d77ce2b3, 0x003457],
    },
    BigNum {
        limbs: [0x32824078eaa4fe8dd702cb688bc087, 0x405d25a31660a733b23a98ca5b22a0, 0x003f96],
    },
    BigNum {
        limbs: [0x74903694b04fd86037fe81ae99502e, 0xb22e5ce11044babc5affca86bf658e, 0x001333],
    },
    BigNum {
        limbs: [0x4913cc6b4e0c269fc8017d5166afd3, 0xf524ccbc6d03787d7d083f1b189fc5, 0x0060b9],
    },
];

global NEGATIVE_ROOTS = [
    BigNum {
        limbs: [0xbda402fffe5bfeffffffff00000000, 0xa753299d7d483339d80809a1d80553, 0x0073ed],
    },
    BigNum {
        limbs: [0x000000000000000000000000000001, 0x000000000000000000000000000000, 0x000000],
    },
    BigNum {
        limbs: [0xbaa40089fb5bfefffeffff00000001, 0xa753299d7d47a5e80b39939ed33467, 0x0073ed],
    },
    BigNum {
        limbs: [0x030002760300000001000000000000, 0x0000000000008d51ccce760304d0ec, 0x000000],
    },
    BigNum {
        limbs: [0x32824078eaa4fe8dd702cb688bc087, 0x405d25a31660a733b23a98ca5b22a0, 0x003f96],
    },
    BigNum {
        limbs: [0x8b21c28713b7007228fd3397743f7a, 0x66f603fa66e78c0625cd70d77ce2b3, 0x003457],
    },
    BigNum {
        limbs: [0x4913cc6b4e0c269fc8017d5166afd3, 0xf524ccbc6d03787d7d083f1b189fc5, 0x0060b9],
    },
    BigNum {
        limbs: [0x74903694b04fd86037fe81ae99502e, 0xb22e5ce11044babc5affca86bf658e, 0x001333],
    },
];
