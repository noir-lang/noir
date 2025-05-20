use bigint::{bignum::BigNum, fields::bls12_381Fr::BLS12_381_Fr as F};
use types::constants::FIELDS_PER_BLOB;

// TODO(#9982): Delete this file and go back to using config.nr - calculating ROOTS in unconstrained is insecure.

pub global LOG_FIELDS_PER_BLOB: u32 = 12;
pub global EXTRA_FIELDS_PER_BLOB: u32 = 16; // 16 = floor(4096 FIELDS_PER_BLOB / 254 noir_field_bits), wasting only 32 bits.
pub global NOIR_FIELDS_PER_BLOB: u32 = FIELDS_PER_BLOB + EXTRA_FIELDS_PER_BLOB;
pub global FIELDS_CARRYING_AN_EXTRA_BIT_PER_BLOB: u32 = EXTRA_FIELDS_PER_BLOB * 254; // EXTRA_FIELDS_PER_BLOB * 254 = 4064. So the first 4064 bls Fr fields in the blob will carry an extra bit in their 255th bit position, that will be used to reconstitute 16 extra fields.
pub global D: F = F::from_limbs([4096, 0, 0]);
pub global D_INV: F = F::from_limbs([
    0x686828bfce5c19400fffff00100001,
    0x6878b46ae3705eb6a46a89213de7d3,
    0x73e6,
]);

unconstrained fn compute_level(idx_: u32) -> u32 {
    // Count the number of trailing ones.
    let mut count = 0;
    let mut idx = idx_;
    for _i in 0..LOG_FIELDS_PER_BLOB {
        if (idx & 1 == 0) {
            break;
        } else {
            count += 1;
            idx >>= 1;
        }
    }

    count
}

unconstrained fn compute_big_minus_arr(
    _big_minus: u32,
    _next_diff: u32,
) -> [u32; LOG_FIELDS_PER_BLOB - 2] {
    let mut res: [u32; LOG_FIELDS_PER_BLOB - 2] = [0; LOG_FIELDS_PER_BLOB - 2];
    let mut big_minus = _big_minus;
    let mut next_diff = _next_diff;
    res[0] = big_minus;
    for i in 1..LOG_FIELDS_PER_BLOB - 2 {
        next_diff >>= 1;
        big_minus += next_diff;
        res[i] = big_minus;
    }
    assert(next_diff == 3);
    res
}

unconstrained fn bit_reversal_permutation(arr: [F; FIELDS_PER_BLOB]) -> [F; FIELDS_PER_BLOB] {
    let mut arr_brp: [F; FIELDS_PER_BLOB] = [BigNum::zero(); FIELDS_PER_BLOB];
    let PLUS = FIELDS_PER_BLOB >> 1;
    let MINUS = PLUS >> 1;
    let mut I = 0;
    let mut next_diff = PLUS - (MINUS >> 1);
    let mut big_minus = next_diff + MINUS;

    let big_minus_arr = compute_big_minus_arr(big_minus, next_diff);

    arr_brp[0] = arr[0];
    I += PLUS;
    arr_brp[1] = arr[I];
    I -= MINUS;
    arr_brp[2] = arr[I];
    I += PLUS;
    arr_brp[3] = arr[I];

    for i in 0..FIELDS_PER_BLOB / 4 - 1 {
        let j = 4 * i + 4;
        let level = compute_level(i);
        I -= big_minus_arr[level];
        arr_brp[j] = arr[I];
        I += PLUS;
        arr_brp[j + 1] = arr[I];
        I -= MINUS;
        arr_brp[j + 2] = arr[I];
        I += PLUS;
        arr_brp[j + 3] = arr[I];
    }
    arr_brp
}

// x ^ i for i in 0..4096
unconstrained fn compute_powers(x: F) -> [F; FIELDS_PER_BLOB] {
    let mut powers: [F; FIELDS_PER_BLOB] = [F::zero(); FIELDS_PER_BLOB];
    let mut current_power: F = F::one();
    for i in 0..FIELDS_PER_BLOB {
        powers[i] = current_power;
        current_power = current_power.__mul(x);
    }
    powers
}

pub unconstrained fn compute_roots_of_unity() -> [F; FIELDS_PER_BLOB] {
    // The below const is found from:
    // let order: F = BigNum { limbs: [ FIELDS_PER_BLOB, 0, 0 ];
    // let exp = BLS_MODULUS_SUB_1.udiv_mod(order);
    // let exp: F = BigNum { limbs: [ 0x553bda402fffe5bfeffffffff00000, 0x3eda753299d7d483339d80809a1d80, 0x000007 ] }; // (MODULUS - 1) // 4096
    // let root_of_unity = PRIMITIVE_ROOT_OF_UNITY.__pow(exp);
    let root_of_unity: F = F::from_limbs([
        0xd1347b378fbf96e206da11a5d36306,
        0x0a11a0f704f4fc3e8acfe0f8245f0a,
        0x00564c,
    ]);
    let mut roots_of_unity = compute_powers(root_of_unity);
    bit_reversal_permutation(roots_of_unity)
    // bit_reversal_permutation(&mut roots_of_unity);
    // roots_of_unity
}

mod tests {
    #[test]
    unconstrained fn test_roots_correctness() {
        assert_eq(super::compute_roots_of_unity(), crate::config::ROOTS);
    }
}
