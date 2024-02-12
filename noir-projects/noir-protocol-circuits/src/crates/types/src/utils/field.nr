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

// TODO to radix returns u8, so we cannot use bigger radixes. It'd be ideal to use a radix of the maximum range-constrained integer noir supports
pub fn full_field_less_than(lhs: Field, rhs: Field) -> bool {
    lhs.lt(rhs)
}

pub fn full_field_greater_than(lhs: Field, rhs: Field) -> bool {
    rhs.lt(lhs)
}
