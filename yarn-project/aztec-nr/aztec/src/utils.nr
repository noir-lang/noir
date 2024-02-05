pub fn arr_copy_slice<T, N, M>(src: [T; N], mut dst: [T; M], offset: Field) -> [T; M] {
    for i in 0..dst.len() {
        dst[i] = src[i + offset];
    }
    dst
}

// TODO(#3470): Copied over from https://github.com/AztecProtocol/aztec-packages/blob/a07c4bd47313be6aa604a63f37857eb0136b41ba/yarn-project/noir-protocol-circuits/src/crates/rollup-lib/src/base/base_rollup_inputs.nr#L599
// move to a shared place?

// TODO to radix returns u8, so we cannot use bigger radixes. It'd be ideal to use a radix of the maximum range-constrained integer noir supports
pub fn full_field_less_than(lhs: Field, rhs: Field) -> bool {
    lhs.lt(rhs)
}

pub fn full_field_greater_than(lhs: Field, rhs: Field) -> bool {
    rhs.lt(lhs)
}
