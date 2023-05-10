use dep::std;

// This is not fully supported by any backend
// Barretenberg backend will not verify a proof, even though it
// is below the WASM limit.
//
// This is here for two reasons:
// - To check for API regressions.
// - Noir should be agnostic to backends, so even though
// the only backend implemented does not work properly for this,
// we can still implement it on the frontend.
fn main(message : [u8;38], pub_key_x : [u8;32], pub_key_y : [u8;32], signature : [u8;64]) {
     // Is there ever a situation where someone would want 
     // to ensure that a signature was invalid?
     let hashed_message = std::hash::sha256(message);
     let x = std::ecdsa_secp256k1::verify_signature(pub_key_x, pub_key_y, signature, hashed_message);
     assert(x == 1);
}