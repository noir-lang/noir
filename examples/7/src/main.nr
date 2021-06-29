// This is the same as Blake2s example.
//
// Pre-alpha dependencies must now be prefixed with the word "dep".
// The line below indicates that we would like to pull in the standard library dependency.

use dep::std;

fn main(x : [5]u8, result : [32]u8) {
     let digest = std::hash::blake2s(x);
     constrain digest == result;
}