use dep::std;

fn main(x: [u8; 5], result: [u8; 32]) {
    let digest = std::hash::blake3_array(x);
    assert(digest == result);
}
