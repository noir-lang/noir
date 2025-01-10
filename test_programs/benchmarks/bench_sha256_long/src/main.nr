// Input size long enough that we have to compress a few times
// and then pad the last block out.
global INPUT_SIZE: u32 = 2 * 64 + 60;

fn main(input: [u8; INPUT_SIZE]) -> pub [u8; 32] {
    std::hash::sha256(input)
}
