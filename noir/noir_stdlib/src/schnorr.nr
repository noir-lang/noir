#[foreign(schnorr_verify)]
// docs:start:schnorr_verify
pub fn verify_signature<N>(
    _public_key_x: Field,
    _public_key_y: Field,
    _signature: [u8; 64],
    _message: [u8; N]
) -> bool
// docs:end:schnorr_verify
{}
