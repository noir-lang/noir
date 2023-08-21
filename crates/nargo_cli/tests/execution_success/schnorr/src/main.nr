// Note: If main has any unsized types, then the verifier will never be able
// to figure out the circuit instance

use dep::std;

fn main(message: [u8; 10], pub_key_x: Field, pub_key_y: Field, signature: [u8; 64]) {
    // Is there ever a situation where someone would want 
    // to ensure that a signature was invalid?
    let valid_signature = std::schnorr::verify_signature(pub_key_x,pub_key_y,signature, message);
    assert(valid_signature);
}
