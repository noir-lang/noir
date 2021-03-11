


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
fn main(hashed_message : [32]u8, pub_key_x : [32]u8, pub_key_y : [32]u8, signature : [64]u8) {

     // Is there ever a situation where someone would want 
     // to ensure that a signature was invalid?
     priv x = std::ecdsa_secp256k1::verify_signature(signature, hashed_message,pub_key_x,pub_key_y);
     
     constrain x == 1;
}