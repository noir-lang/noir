// docs:start:ecdsa_secp256r1
pub fn verify_signature(
    public_key_x: [u8; 32],
    public_key_y: [u8; 32],
    signature: [u8; 64],
    message_hash: [u8; 32],
) -> bool
// docs:end:ecdsa_secp256r1
{
    _verify_signature(public_key_x, public_key_y, signature, message_hash, true)
}

#[foreign(ecdsa_secp256r1)]
fn _verify_signature(
    public_key_x: [u8; 32],
    public_key_y: [u8; 32],
    signature: [u8; 64],
    message_hash: [u8; 32],
    predicate: bool,
) -> bool {}
