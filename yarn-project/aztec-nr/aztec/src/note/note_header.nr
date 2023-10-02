struct NoteHeader {
    contract_address: Field,
    nonce: Field,
    storage_slot: Field,
    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1386)
    // Remove this and check the nonce to see whether a note is transient or not.
    is_transient: bool,
}

impl NoteHeader {
    fn new(contract_address: Field, nonce: Field, storage_slot: Field) -> Self {
        NoteHeader { contract_address, nonce, storage_slot, is_transient: false }
    }

    fn empty() -> Self {
        NoteHeader { contract_address: 0, nonce: 0, storage_slot: 0, is_transient: false }
    }
}