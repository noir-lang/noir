
// Sha256 circuit where the input is 5 bytes
// not five field elements since sha256 operates over
// bytes.
//
// If you do not cast, it will take all the bytes from the field element!

use dep::std;

fn main(x : [5]u8, low : Public, high: Public) {

     let digest = std::hash::sha256(x);
     
     constrain digest[0] == low;
     constrain digest[1] == high;
}