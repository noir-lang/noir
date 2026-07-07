fn main(
    pub_key_x: [u8; 32],
    pub_key_y: [u8; 32],
    signature: [u8; 64], // r || s
    hashed_message: [u8; 32],
    active: bool,
) {
    if active {
        assert(std::ecdsa_secp256r1::verify_signature(
            pub_key_x,
            pub_key_y,
            signature,
            hashed_message,
        ));
    }
}
