
// Sha256 circuit where the input is 5 bytes
// not five field elements since sha256 operates over
// bytes.
//
// If you do not cast, it will take all the bytes from the field element!

use dep::std;

fn main(x : [5]u8, result : pub [32]u8) {

     let digest = std::hash::sha256(x);
     
     constrain digest == result;
}
