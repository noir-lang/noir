use dep::protocol_types::{address::AztecAddress, grumpkin_point::GrumpkinPoint};

// = 480 + 32 * N bytes
#[oracle(emitEncryptedNoteLog)]
fn emit_encrypted_note_log_oracle<M>(_note_hash: Field, _encrypted_note: [u8; M], _counter: u32) {}

unconstrained pub fn emit_encrypted_note_log<M>(
    note_hash: Field,
    encrypted_note: [u8; M],
    counter: u32
) {
    emit_encrypted_note_log_oracle(note_hash, encrypted_note, counter)
}

#[oracle(emitEncryptedLog)]
fn emit_encrypted_log_oracle<M>(_encrypted_note: [u8; M], _counter: u32) {}

unconstrained pub fn emit_encrypted_log<M>(encrypted_note: [u8; M], counter: u32) {
    emit_encrypted_log_oracle(encrypted_note, counter)
}

// = 480 + 32 * N bytes
#[oracle(computeEncryptedLog)]
fn compute_encrypted_log_oracle<N, M>(
    _contract_address: AztecAddress,
    _storage_slot: Field,
    _note_type_id: Field,
    _encryption_pub_key: GrumpkinPoint,
    _preimage: [Field; N]
) -> [u8; M] {}

unconstrained pub fn compute_encrypted_log<N, M>(
    contract_address: AztecAddress,
    storage_slot: Field,
    note_type_id: Field,
    ivpk_m: GrumpkinPoint,
    preimage: [Field; N]
) -> [u8; M] {
    compute_encrypted_log_oracle(contract_address, storage_slot, note_type_id, ivpk_m, preimage)
}
