#[foreign(ecdsa_secp256k1)]
// docs:start:ecdsa_secp256k1
pub fn verify_signature<N>(
    _public_key_x: [u8; 32],
    _public_key_y: [u8; 32],
    _signature: [u8; 64],
    _message_hash: [u8; N]
) -> bool
// docs:end:ecdsa_secp256k1
{}
