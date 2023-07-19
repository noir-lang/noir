struct NoteHeader {
    contract_address: Field,
    nonce: Field,
    storage_slot: Field,
}

impl NoteHeader {
    fn empty() -> Self {
        NoteHeader { contract_address: 0, nonce: 0, storage_slot: 0 }
    }
}