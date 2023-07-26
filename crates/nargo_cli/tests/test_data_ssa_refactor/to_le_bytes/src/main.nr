use dep::std;

fn main(x : Field) -> pub [u8; 4] {
    // The result of this byte array will be little-endian
    let byte_array = x.to_le_bytes(31);
    let mut first_four_bytes = [0; 4];
    for i in 0..4 {
        first_four_bytes[i] = byte_array[i];
    }

    let chash = std::hash::keccak256(x.to_le_bytes(8), 8);
    assert(chash[0] > 0 | chash[1] > 0);

    // Issue #617 fix
    // We were incorrectly mapping our output array from bit decomposition functions during acir generation
    first_four_bytes[3] = byte_array[31];
    first_four_bytes
}