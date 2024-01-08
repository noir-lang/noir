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
