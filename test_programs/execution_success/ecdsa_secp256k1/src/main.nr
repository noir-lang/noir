fn main(
    message: [u8; 38],
    hashed_message: [u8; 32],
    pub_key_x: [u8; 32],
    pub_key_y: [u8; 32],
    signature: [u8; 64],
) {
    // Hash the message, since secp256k1 expects a hashed_message
    let expected = std::hash::sha256(message);
    assert(hashed_message == expected);

    let valid_signature =
        std::ecdsa_secp256k1::verify_signature(pub_key_x, pub_key_y, signature, hashed_message);
    assert(valid_signature);
}
