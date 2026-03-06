struct Signature {
    message_hash: [u8; 32],
    pub_key_x: [u8; 32],
    pub_key_y: [u8; 32],
    signature: [u8; 64],
}

fn main(invalid_signatures: [Signature; 3]) {
    for signature in invalid_signatures {
        let is_valid = std::ecdsa_secp256k1::verify_signature(
            signature.pub_key_x,
            signature.pub_key_y,
            signature.signature,
            signature.message_hash,
        );
        assert(!is_valid);
    }
}
