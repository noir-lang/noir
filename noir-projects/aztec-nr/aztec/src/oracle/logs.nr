use dep::protocol_types::{address::AztecAddress, constants::NUM_FIELDS_PER_SHA256, grumpkin_point::GrumpkinPoint};

// TODO: Should take encrypted data.
#[oracle(emitEncryptedLog)]
fn emit_encrypted_log_oracle<N>(
    _contract_address: AztecAddress,
    _storage_slot: Field,
    _note_type_id: Field,
    _encryption_pub_key: GrumpkinPoint,
    _preimage: [Field; N]
) -> Field {}

unconstrained pub fn emit_encrypted_log<N>(
    contract_address: AztecAddress,
    storage_slot: Field,
    note_type_id: Field,
    encryption_pub_key: GrumpkinPoint,
    preimage: [Field; N]
) -> [Field; NUM_FIELDS_PER_SHA256] {
    [
        emit_encrypted_log_oracle(
            contract_address,
            storage_slot,
            note_type_id,
            encryption_pub_key,
            preimage
        )
    ]
}

#[oracle(emitUnencryptedLog)]
fn emit_unencrypted_log_oracle<T>(
    _contract_address: AztecAddress,
    _event_selector: Field,
    _message: T
) -> Field {}

unconstrained pub fn emit_unencrypted_log<T>(
    contract_address: AztecAddress,
    event_selector: Field,
    message: T
) -> [Field; NUM_FIELDS_PER_SHA256] {
    // https://github.com/AztecProtocol/aztec-packages/issues/885
    [emit_unencrypted_log_oracle(contract_address, event_selector, message)]
}
