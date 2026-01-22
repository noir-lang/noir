use crate::BlackBoxResolutionError;
use libaes::Cipher; // cSpell:disable-line

pub fn aes128_encrypt(
    inputs: &[u8],
    iv: [u8; 16],
    key: [u8; 16],
) -> Result<Vec<u8>, BlackBoxResolutionError> {
    if inputs.len() % 16 != 0 {
        return Err(BlackBoxResolutionError::Failed(
            acir::BlackBoxFunc::AES128Encrypt,
            "input length must be a multiple of 16".to_string(),
        ));
    }

    let mut cipher = Cipher::new_128(&key);
    // Disable auto padding - input must be block-aligned (multiple of 16)
    // and output will be the same size as input.
    // See: https://github.com/keepsimple1/libaes/blob/e45afaa1e9f248375e797a52eaf40eeb0ba8515a/tests/aes.rs#L325
    cipher.set_auto_padding(false);
    let encrypted = cipher.cbc_encrypt(&iv, inputs);

    if encrypted.is_empty() && !inputs.is_empty() {
        return Err(BlackBoxResolutionError::Failed(
            acir::BlackBoxFunc::AES128Encrypt,
            "AES encryption failed - input may not be block-aligned".to_string(),
        ));
    }

    Ok(encrypted)
}
