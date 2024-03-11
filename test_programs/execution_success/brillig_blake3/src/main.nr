use dep::std;

unconstrained fn main(x: [u8; 5], result: [u8; 32]) {
    let digest = std::hash::blake3(x.as_slice());
    assert(digest == result);
}
