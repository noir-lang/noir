// blake3 circuit where the input is 5 bytes
// not five field elements since blake3 operates over
// bytes.
//
// If you do not cast, it will take all the bytes from the field element!
fn main(x: [u8; 5], result: pub [u8; 32]) {
    let mut digest = std::hash::blake3(x);
    digest[0] = 5u8;
    digest = std::hash::blake3(x);
    assert(digest == result);
}
