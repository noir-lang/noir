use dep::std;

fn main(x: [u8; 32], result: [u8; 32]) {
    let digest = std::hash::sha256(x);
    assert(digest == result);
}
