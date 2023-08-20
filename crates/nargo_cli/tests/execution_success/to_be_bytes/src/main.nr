use dep::std;

fn main(x : Field) -> pub [u8; 31] {
    // The result of this byte array will be big-endian
    let byte_array = x.to_be_bytes(31);
    let mut bytes = [0; 31];
    for i in 0..31 {
        bytes[i] = byte_array[i];
    }
    std::assert_eq(bytes[30], 60);
    std::assert_eq(bytes[29], 33);
    std::assert_eq(bytes[28], 31);
    bytes
}
