
fn main(input: [u8; 2]) -> pub [u8; 32] {
    std::hash::sha256(input)
}