struct NoteHeader {
    contract_address: Field,
    storage_slot: Field,
    // nonce: Field,
}

impl NoteHeader {
    fn empty() -> Self {
        NoteHeader { contract_address: 0, storage_slot: 0 }
    }
}