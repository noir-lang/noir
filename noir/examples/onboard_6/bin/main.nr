

use dep::std;

// Notice that if main had []Witness, 
// The verifier would never be able to figure out the
// circuit instance!
fn main(message : [10]u8, pub_key_x : Witness, pub_key_y : Witness,signature : [64]u8) {

     const zero = 0;

     // Is there ever a situation where someone would want 
     // to ensure that a signature was invalid?
     std::schnorr::verify_signature(signature, message,pub_key_x,pub_key_y);
     
     // Type is extracted from the last statement
     constrain message[0] == 0;
}