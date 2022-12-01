use dep::std;

fn main(x : Field) -> pub [u8; 4] {
    // The result of this byte array will be little-endian
    let byte_array = std::to_bytes(x, 253);
    let mut first_four_bytes = [0; 4];
    for i in 0..4 {
        first_four_bytes[i] = byte_array[i];
    }
    first_four_bytes
}