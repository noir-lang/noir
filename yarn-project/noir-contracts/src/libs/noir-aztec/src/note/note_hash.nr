fn compute_siloed_note_hash(storage_slot: Field, note_hash: Field) -> Field {
    dep::std::hash::pedersen([
        storage_slot,
        note_hash,
    ])[0]
}

// TODO: Should include contract address and nonce.
fn compute_final_note_hash(storage_slot: Field, note_hash: Field) -> Field {
    dep::std::hash::pedersen([
        storage_slot,
        note_hash,
    ])[0]
}