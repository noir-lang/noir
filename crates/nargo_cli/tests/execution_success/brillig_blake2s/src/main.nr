// Tests a very simple program.
// 
// The features being tested is blake2s in brillig

use dep::std;

fn main(x: [u8; 5], result: [u8; 32]) {
    assert(blake2s(x) == result);
}

unconstrained fn blake2s(x: [u8; 5]) -> [u8; 32] {
    std::hash::blake2s(x)
}
