use dep::protocol_types::{address::AztecAddress, grumpkin_point::GrumpkinPoint};

// TODO(1139): Should take encrypted data.
// Currently returns encrypted data to be hashed
// = 112 + 32 * (N + 3) bytes = N + 7 fields
#[oracle(emitEncryptedLog)]
fn emit_encrypted_log_oracle<N, M>(
    _contract_address: AztecAddress,
    _storage_slot: Field,
    _note_type_id: Field,
    _encryption_pub_key: GrumpkinPoint,
    _preimage: [Field; N],
    _counter: u32
) -> [Field; M] {}

unconstrained pub fn emit_encrypted_log<N, M>(
    contract_address: AztecAddress,
    storage_slot: Field,
    note_type_id: Field,
    ivpk_m: GrumpkinPoint,
    preimage: [Field; N],
    counter: u32
) -> [Field; M] {
    emit_encrypted_log_oracle(
        contract_address,
        storage_slot,
        note_type_id,
        ivpk_m,
        preimage,
        counter
    )
}
