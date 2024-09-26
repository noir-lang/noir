use dep::std;

// Tests that the sha256 digest works. This fails, because SHA256 opcode was
// removed and SHA256COMPRESSION was introduced, which does not easily replace
// it.
fn main(x: [u8; 32], result: [u8; 32]) {
    let digest = std::hash::sha256(x);
    assert(digest == result);
}
