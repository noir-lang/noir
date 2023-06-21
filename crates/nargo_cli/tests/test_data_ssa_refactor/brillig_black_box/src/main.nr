use dep::std;

// Tests a very simple program.
// 
// The features being tested are blackbox operations on brillig
fn main(msg_bytes: [u8; 32]) -> pub [u8; 32] {
    sha256(msg_bytes)
}

unconstrained fn sha256(x: [u8; 32]) -> [u8; 32] {
    std::hash::sha256(x)
}

// unconstrained fn verify_signature() {
//     assert(std::ecdsa_secp256k1::verify_signature([0; 32], [0; 32], [0; 64], [0; 32]));
// }