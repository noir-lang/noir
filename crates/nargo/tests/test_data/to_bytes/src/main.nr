use dep::std;

fn main(x : Field) -> pub [u8; 4] {
    // The result of this byte array will be little-endian
    let byte_array = std::to_bytes(x, 31);
    let mut first_four_bytes = [0; 4];
    for i in 0..4 {
        first_four_bytes[i] = byte_array[i];
    }
    // Issue #617 fix
    // We were incorrectly mapping our output array from bit decomposition functions during acir generation
    first_four_bytes[3] = byte_array[31];
    first_four_bytes
}