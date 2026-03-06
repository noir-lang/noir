use crate::BlackBoxResolutionError;

mod secp256k1;
mod secp256r1;

pub fn ecdsa_secp256k1_verify(
    hashed_msg: &[u8; 32],
    public_key_x: &[u8; 32],
    public_key_y: &[u8; 32],
    signature: &[u8; 64],
) -> Result<bool, BlackBoxResolutionError> {
    secp256k1::verify_signature(hashed_msg, public_key_x, public_key_y, signature)
}

pub fn ecdsa_secp256r1_verify(
    hashed_msg: &[u8; 32],
    public_key_x: &[u8; 32],
    public_key_y: &[u8; 32],
    signature: &[u8; 64],
) -> Result<bool, BlackBoxResolutionError> {
    secp256r1::verify_signature(hashed_msg, public_key_x, public_key_y, signature)
}
