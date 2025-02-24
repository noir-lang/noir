// Test Noir implementations of SHA256 and SHA512 on a one-byte message.
fn main(x: Field, result256: [u8; 32], result512: [u8; 64]) {
    let digest256 = std::hash::sha256([x as u8]);
    assert(digest256 == result256);

    let digest512 = std::hash::sha512::digest([x as u8]);
    assert(digest512 == result512);
}
