use dep::std;

// Note: If main has any unsized types, then the verifier will never be able
// to figure out the circuit instance
fn main(message : [10]u8, pub_key_x : Field, pub_key_y : Field, signature : [64]u8) {
     // Is there ever a situation where someone would want 
     // to ensure that a signature was invalid?
      let x = std::schnorr::verify_signature(pub_key_x,pub_key_y,signature, message);
     constrain x == 1;
}
