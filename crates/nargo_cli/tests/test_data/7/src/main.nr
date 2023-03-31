// This is the same as Blake2s example.
//
// Pre-alpha dependencies must now be prefixed with the word "dep".
// The line below indicates that we would like to pull in the standard library dependency.
use dep::std;

fn main(x: [u8; 5], result: [u8; 32]) {
    let digest = std::hash::blake2s(x);
    constrain digest == result;
}
