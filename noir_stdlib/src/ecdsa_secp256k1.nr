#[foreign(ecdsa_secp256k1)]
// docs:start:ecdsa_secp256k1
pub fn verify_signature<N>(
    public_key_x: [u8; 32],
    public_key_y: [u8; 32],
    signature: [u8; 64],
    message_hash: [u8; N]
) -> bool
// docs:end:ecdsa_secp256k1
{}
