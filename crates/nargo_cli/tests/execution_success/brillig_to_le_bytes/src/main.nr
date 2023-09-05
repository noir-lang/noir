unconstrained fn main(x : Field) -> pub [u8; 31] {
    // The result of this byte array will be little-endian
    let byte_array = x.to_le_bytes(31);
    assert(byte_array.len() == 31);
    let mut bytes = [0; 31];
    for i in 0..31 {
        bytes[i] = byte_array[i];
    }
    bytes
}
