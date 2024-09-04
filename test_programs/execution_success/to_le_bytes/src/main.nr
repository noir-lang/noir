fn main(x: Field, cond: bool) -> pub [u8; 31] {
    // The result of this byte array will be little-endian
    let byte_array: [u8; 31] = x.to_le_bytes();
    assert(byte_array.len() == 31);

    let mut bytes = [0; 31];
    for i in 0..31 {
        bytes[i] = byte_array[i];
    }

    if cond {
        // We've set x = "2040124" so we shouldn't be able to represent this as a single byte.
        let bad_byte_array: [u8; 1] = x.to_le_bytes();
        assert_eq(bad_byte_array.len(), 1);
    }

    bytes
}
