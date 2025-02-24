fn main(
    hashed_message: [u8; 32],
    pub_key_x: [u8; 32],
    pub_key_y: [u8; 32],
    signature: [u8; 64],
    hashed_message_2: [u8; 32],
    pub_key_x_2: [u8; 32],
    pub_key_y_2: [u8; 32],
    signature_2: [u8; 64],
    hashed_message_3: [u8; 32],
    pub_key_x_3: [u8; 32],
    pub_key_y_3: [u8; 32],
    signature_3: [u8; 64],
) {
    let valid_signature =
        std::ecdsa_secp256r1::verify_signature(pub_key_x, pub_key_y, signature, hashed_message);
    assert(valid_signature);
    let valid_signature_2 = std::ecdsa_secp256r1::verify_signature(
        pub_key_x_2,
        pub_key_y_2,
        signature_2,
        hashed_message_2,
    );
    assert(valid_signature_2);
    let valid_signature_3 = std::ecdsa_secp256r1::verify_signature(
        pub_key_x_3,
        pub_key_y_3,
        signature_3,
        hashed_message_3,
    );
    assert(valid_signature_3);
}
