#[foreign(ecdsa_secp256r1)]
// docs:start:ecdsa_secp256r1
pub fn verify_signature<let N: u32>(
    public_key_x: [u8; 32],
    public_key_y: [u8; 32],
    signature: [u8; 64],
    message_hash: [u8; N],
) -> bool
// docs:end:ecdsa_secp256r1
{}
