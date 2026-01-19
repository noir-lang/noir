use crate::BlackBoxResolutionError;
use libaes::Cipher; // cSpell:disable-line

pub fn aes128_encrypt(
    inputs: &[u8],
    iv: [u8; 16],
    key: [u8; 16],
) -> Result<Vec<u8>, BlackBoxResolutionError> {
    let cipher = Cipher::new_128(&key);
    let encrypted = cipher.cbc_encrypt(&iv, inputs);
    Ok(encrypted)
}
