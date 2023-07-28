use dep::std;

// Tests a very simple program.
// 
// The features being tested is ecdsa in brillig
fn main(hashed_message : [u8;32], pub_key_x : [u8;32], pub_key_y : [u8;32], signature : [u8;64]) {
    assert(ecdsa(hashed_message, pub_key_x, pub_key_y, signature));
}

unconstrained fn ecdsa(hashed_message : [u8;32], pub_key_x : [u8;32], pub_key_y : [u8;32], signature : [u8;64]) -> bool {
    std::ecdsa_secp256k1::verify_signature(pub_key_x, pub_key_y, signature, hashed_message)
}