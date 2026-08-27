struct Signature {
    message_hash: [u8; 32],
    pub_key_x: [u8; 32],
    pub_key_y: [u8; 32],
    signature: [u8; 64],
}

// `ecdsa_secp256k1::verify_signature` is given a predicate by the compiler so that a call
// sitting in a branch which is not taken is not constrained. The three signatures below are
// malformed (r = 0, s = 0, and both), shapes which `ecdsa_secp256k1_invalid_inputs` only
// exercises with the call live. Here they must be a don't-care instead, so the assertion
// is that they do verify.
fn main(invalid_signatures: [Signature; 3], active: bool) {
    if active {
        for signature in invalid_signatures {
            assert(std::ecdsa_secp256k1::verify_signature(
                signature.pub_key_x,
                signature.pub_key_y,
                signature.signature,
                signature.message_hash,
            ));
        }
    }
}
