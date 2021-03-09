// This is the same as Blake2s example.
//
// XXX: 256 bits can fit into the field element representation
// If we allow deserialising from non-canonical bytes 
// However, one reason to not allow this is because the reduced 
// representation will not be the hash.
// Why might be confusing? 

// Pre-alpha dependencies must now be prefixed with the word "dep".
// The line below indicates that we would like to pull in the standard library dependency.

use dep::std;

// XXX(Justin): Constrain array to be equal to a byte array
// implicitly constrain low and high 128 bits 

fn main(x : [5]u8, low : pub Field, high: pub Field) {

     let digest = std::hash::blake2s(x);
     
     constrain digest[0] == low;
     constrain digest[1] == high;
}