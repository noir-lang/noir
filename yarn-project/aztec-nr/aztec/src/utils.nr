pub fn arr_copy_slice<T, N, M>(src: [T; N], mut dst: [T; M], offset: Field) -> [T; M] {
    for i in 0..dst.len() {
        dst[i] = src[i + offset];
    }
    dst
}

pub fn field_from_bytes<N>(bytes: [u8; N], big_endian: bool) -> Field {
    assert(bytes.len() as u32 < 32, "field_from_bytes: N must be less than 32");
    let mut as_field = 0;
    let mut offset = 1;
    for i in 0..N {
        let mut index = i;
        if big_endian {
            index = N - i - 1;
        }
        as_field += (bytes[index] as Field) * offset;
        offset *= 256;
    }

    as_field
}

// TODO(#3470): Copied over from https://github.com/AztecProtocol/aztec-packages/blob/a07c4bd47313be6aa604a63f37857eb0136b41ba/yarn-project/noir-protocol-circuits/src/crates/rollup-lib/src/base/base_rollup_inputs.nr#L599
// move to a shared place?

// TODO to radix returns u8, so we cannot use bigger radixes. It'd be ideal to use a radix of the maximum range-constrained integer noir supports
pub fn full_field_less_than(lhs: Field, rhs: Field) -> bool {
    dep::std::eddsa::lt_bytes32(lhs, rhs)
}

pub fn full_field_greater_than(lhs: Field, rhs: Field) -> bool {
    dep::std::eddsa::lt_bytes32(rhs, lhs)
}