use crate::types::point::Point;
use crate::notes::value_note::Note;

// TODO: Make independent of a specific note implementation.
#[oracle(emitEncryptedLog)]
fn emit_encrypted_log_oracle(
    _contract_address: Field,
    _storage_slot: Field,
    _owner: Point,
    _note: Note,
)  -> Field {}

unconstrained fn emit_encrypted_log(
    contract_address: Field,
    storage_slot: Field,
    owner: Point,
    note: Note,
) -> Field {
    emit_encrypted_log_oracle(
        contract_address, 
        storage_slot,
        owner,
        note,
    )
}