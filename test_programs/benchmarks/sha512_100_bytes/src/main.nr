fn main(input: [u8; 100]) -> pub [u8; 64] {
    sha512::sha512::digest(input)
}
