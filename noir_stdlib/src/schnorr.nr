#[foreign(schnorr_verify)]
// docs:start:schnorr_verify
pub fn verify_signature(
    public_key_x: Field,
    public_key_y: Field,
    signature: [u8; 64],
    message: [u8]
) -> bool
// docs:end:schnorr_verify
{}
