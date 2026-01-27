fn main(
    hashed_message: [u8; 32],
    pub_key_x: [u8; 32],
    pub_key_y: [u8; 32],
    signature: [u8; 64],
    hashed_message_2: [u8; 32],
    pub_key_x_2: [u8; 32],
    pub_key_y_2: [u8; 32],
    signature_2: [u8; 64],
) {
    // First test: message equals the order of the curve
    let valid_signature =
        std::ecdsa_secp256k1::verify_signature(pub_key_x, pub_key_y, signature, hashed_message);
    assert(!valid_signature);

    // Second test: R.x >= curve order
    // signature with r = s = 1 and message = 1
    // Hence, in ecdsa verification, u1 and u2 are 1, so R = G (generator) + P (pubkey)
    // P is chosen so that P = R - G, with R.x = curve's field modulus - 3
    let valid_signature = std::ecdsa_secp256k1::verify_signature(
        pub_key_x_2,
        pub_key_y_2,
        signature_2,
        hashed_message_2,
    );
    assert(!valid_signature);
}
