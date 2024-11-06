pub struct U60Repr<let N: u32, let NumSegments: u32> {
    limbs: [u64; N * NumSegments],
}

fn main() {
    let numerator =
        [790096867046896348, 1063071665130103641, 602707730209562162, 996751591622961462, 28650, 0];
    let denominator = [12, 0, 0, 0, 0, 0];

    unsafe { __udiv_mod(numerator) };

    let result = unsafe { __validate_gt_remainder(denominator) };
    assert(result[4] == 0);
    assert(result[5] == 0);
}

unconstrained fn __udiv_mod<let N: u32>(remainder_u60: [u64; 2 * N]) {
    let bit_difference = get_msb(remainder_u60);
    let mut accumulator_u60: U60Repr<N, 2> = U60Repr { limbs: [0; N * 2] };
    accumulator_u60.limbs[0] = 1;
    accumulator_u60 = shl(accumulator_u60, bit_difference);
}

unconstrained fn __validate_gt_remainder<let N: u32>(a_u60: [u64; N]) -> [u64; N] {
    let mut addend_u60: [u64; N] = [0; N];
    let mut result_u60: [u64; N] = [0; N];

    for i in 0..N {
        result_u60[i] = a_u60[i] + addend_u60[i];
    }

    result_u60
}

unconstrained fn get_msb<let N: u32>(val: [u64; N]) -> u32 {
    let mut count = 0;
    for i in 0..N {
        let v = val[(N - 1) - i];
        if (v > 0) {
            count = 60 * ((N - 1) - i);
            break;
        }
    }
    count
}

fn shl<let N: u32>(val: U60Repr<N, 2>, shift: u32) -> U60Repr<N, 2> {
    let mut result: U60Repr<N, 2> = U60Repr { limbs: [0; 2 * N] };

    let num_shifted_limbs = shift / 60;
    let limb_shift = (shift % 60) as u8;

    let mask: u64 = (1 as u64 << 60) - 1;
    let value = val.limbs[0];

    result.limbs[num_shifted_limbs] = (value << limb_shift) & mask;

    for i in 1..((N * 2) - num_shifted_limbs) {
        let value = val.limbs[i];
        let upshift = (value << limb_shift) & mask;
        result.limbs[i + num_shifted_limbs] = upshift;
    }
    result
}
