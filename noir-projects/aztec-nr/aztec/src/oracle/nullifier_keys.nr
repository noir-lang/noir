use dep::protocol_types::{grumpkin_point::GrumpkinPoint, abis::nullifier_key_validation_request::NullifierKeyValidationRequest};

#[oracle(getNullifierKeys)]
fn get_nullifier_key_validation_request_oracle(_npk_m_hash: Field) -> [Field; 3] {}

unconstrained fn get_nullifier_key_validation_request_internal(npk_m_hash: Field) -> NullifierKeyValidationRequest {
    let result = get_nullifier_key_validation_request_oracle(npk_m_hash);
    NullifierKeyValidationRequest {
        master_nullifier_public_key: GrumpkinPoint { x: result[0], y: result[1] },
        app_nullifier_secret_key: result[2]
    }
}

// We get the full struct Nullifier Keys here
pub fn get_nullifier_key_validation_request(npk_m_hash: Field) -> NullifierKeyValidationRequest {
    get_nullifier_key_validation_request_internal(npk_m_hash)
}

pub fn get_nsk_app(npk_m_hash: Field) -> Field {
    get_nullifier_key_validation_request_internal(npk_m_hash).app_nullifier_secret_key
}
