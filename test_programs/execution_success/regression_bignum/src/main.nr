fn main() {
    let numerator =
        [790096867046896348, 1063071665130103641, 602707730209562162, 996751591622961462, 28650, 0];
    unsafe { __udiv_mod(numerator) };

    let denominator = [12, 0, 0, 0, 0, 0];
    let result = unsafe { __validate_gt_remainder(denominator) };
    assert(result[4] == 0);
    assert(result[5] == 0);
}

unconstrained fn __udiv_mod(remainder_u60: [u64; 6]) {
    let bit_difference = get_msb(remainder_u60);
    let accumulator_u60: [u64; 6] = shl(bit_difference);
}

unconstrained fn __validate_gt_remainder(a_u60: [u64; 6]) -> [u64; 6] {
    let mut addend_u60: [u64; 6] = [0; 6];
    let mut result_u60: [u64; 6] = [0; 6];

    for i in 0..6 {
        result_u60[i] = a_u60[i] + addend_u60[i];
    }

    result_u60
}

unconstrained fn get_msb(val: [u64; 6]) -> u32 {
    let mut count = 0;
    for i in 0..6 {
        let v = val[(6 - 1) - i];
        if (v > 0) {
            count = 60 * ((6 - 1) - i);
            break;
        }
    }
    count
}

unconstrained fn shl(shift: u32) -> [u64; 6] {
    let num_shifted_limbs = shift / 60;
    let limb_shift = (shift % 60) as u8;

    let mut result = [0; 6];
    result[num_shifted_limbs] = (1 << limb_shift);

    for i in 1..(6 - num_shifted_limbs) {
        result[i + num_shifted_limbs] = 0;
    }
    result
}
