use dep::protocol_types::{address::AztecAddress, point::Point};

// = 480 + 32 * N bytes
#[oracle(emitEncryptedNoteLog)]
unconstrained fn emit_encrypted_note_log_oracle<M>(_note_hash_counter: u32, _encrypted_note: [u8; M], _counter: u32) {}

unconstrained pub fn emit_encrypted_note_log<M>(
    note_hash_counter: u32,
    encrypted_note: [u8; M],
    counter: u32
) {
    emit_encrypted_note_log_oracle(note_hash_counter, encrypted_note, counter)
}

#[oracle(emitEncryptedEventLog)]
unconstrained fn emit_encrypted_event_log_oracle<M>(
    _contract_address: AztecAddress,
    _randomness: Field,
    _encrypted_event: [u8; M],
    _counter: u32
) {}

unconstrained pub fn emit_encrypted_event_log<M>(
    contract_address: AztecAddress,
    randomness: Field,
    encrypted_event: [u8; M],
    counter: u32
) {
    emit_encrypted_event_log_oracle(contract_address, randomness, encrypted_event, counter)
}

// = 480 + 32 * N bytes
#[oracle(computeEncryptedNoteLog)]
unconstrained fn compute_encrypted_note_log_oracle<N, M>(
    _contract_address: AztecAddress,
    _storage_slot: Field,
    _note_type_id: Field,
    _ovsk_app: Field,
    _ovpk_m: Point,
    _ivpk_m: Point,
    _preimage: [Field; N]
) -> [u8; M] {}

unconstrained pub fn compute_encrypted_note_log<N, M>(
    contract_address: AztecAddress,
    storage_slot: Field,
    note_type_id: Field,
    ovsk_app: Field,
    ovpk_m: Point,
    ivpk_m: Point,
    preimage: [Field; N]
) -> [u8; M] {
    compute_encrypted_note_log_oracle(
        contract_address,
        storage_slot,
        note_type_id,
        ovsk_app,
        ovpk_m,
        ivpk_m,
        preimage
    )
}

// = 480 + 32 * N bytes
#[oracle(computeEncryptedEventLog)]
unconstrained fn compute_encrypted_event_log_oracle<N, M>(
    _contract_address: AztecAddress,
    _randomness: Field,
    _event_type_id: Field,
    _ovsk_app: Field,
    _ovpk_m: Point,
    _ivpk_m: Point,
    _preimage: [Field; N]
) -> [u8; M] {}

unconstrained pub fn compute_encrypted_event_log<N, M>(
    contract_address: AztecAddress,
    randomness: Field,
    event_type_id: Field,
    ovsk_app: Field,
    ovpk_m: Point,
    ivpk_m: Point,
    preimage: [Field; N]
) -> [u8; M] {
    compute_encrypted_event_log_oracle(
        contract_address,
        randomness,
        event_type_id,
        ovsk_app,
        ovpk_m,
        ivpk_m,
        preimage
    )
}

#[oracle(emitUnencryptedLog)]
unconstrained fn emit_unencrypted_log_oracle_private<T>(_contract_address: AztecAddress, _message: T, _counter: u32) -> Field {}

unconstrained pub fn emit_unencrypted_log_private_internal<T>(
    contract_address: AztecAddress,
    message: T,
    counter: u32
) -> Field {
    emit_unencrypted_log_oracle_private(contract_address, message, counter)
}

#[oracle(emitContractClassUnencryptedLog)]
unconstrained fn emit_contract_class_unencrypted_log_private<N>(contract_address: AztecAddress, message: [Field; N], counter: u32) -> Field {}

unconstrained pub fn emit_contract_class_unencrypted_log_private_internal<N>(contract_address: AztecAddress, message: [Field; N], counter: u32) -> Field {
    emit_contract_class_unencrypted_log_private(contract_address, message, counter)
}
