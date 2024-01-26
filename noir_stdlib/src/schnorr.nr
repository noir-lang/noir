#[foreign(schnorr_verify)]
pub fn verify_signature<N>(
    public_key_x: Field,
    public_key_y: Field,
    signature: [u8; 64],
    message: [u8; N]
) -> bool {}
