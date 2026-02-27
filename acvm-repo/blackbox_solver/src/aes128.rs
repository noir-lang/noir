use crate::BlackBoxResolutionError;
use aes::cipher::{BlockEncryptMut, KeyIvInit, block_padding::NoPadding};

type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>; // cSpell:disable-line

pub fn aes128_encrypt(
    inputs: &[u8],
    iv: [u8; 16],
    key: [u8; 16],
) -> Result<Vec<u8>, BlackBoxResolutionError> {
    if !inputs.len().is_multiple_of(16) {
        return Err(BlackBoxResolutionError::Failed(
            acir::BlackBoxFunc::AES128Encrypt,
            "input length must be a multiple of 16".to_string(),
        ));
    }

    let mut buffer = inputs.to_vec();
    // This blackbox does not apply padding; callers must pad inputs in Noir before calling.
    // The expect cannot fail because we verify block-alignment above.
    Aes128CbcEnc::new(&key.into(), &iv.into())
        .encrypt_padded_mut::<NoPadding>(&mut buffer, inputs.len())
        .expect("input length is block-aligned");

    Ok(buffer)
}
