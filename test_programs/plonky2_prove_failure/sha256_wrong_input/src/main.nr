use dep::std;

// Tests that the sha256 digest is some wrong one, which is why test is expected to fail.
fn main(x: [u8; 32], result: [u8; 32]) {
    let digest = std::hash::sha256(x);
    assert(digest == result);
}
