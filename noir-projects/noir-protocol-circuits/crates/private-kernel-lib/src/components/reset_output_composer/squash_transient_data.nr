use dep::types::abis::{note_hash::ScopedNoteHash, nullifier::ScopedNullifier, log_hash::NoteLogHash};

unconstrained pub fn squash_transient_data<M, N, P>(
    note_hashes: [ScopedNoteHash; M],
    nullifiers: [ScopedNullifier; N],
    logs: [NoteLogHash; P],
    transient_nullifier_indexes_for_note_hashes: [u32; M],
    transient_note_hash_indexes_for_nullifiers: [u32; N]
) -> ([ScopedNoteHash; M], [ScopedNullifier; N], [NoteLogHash; P]) {
    let mut propagated_note_hashes = BoundedVec::new();
    for i in 0..note_hashes.len() {
        if transient_nullifier_indexes_for_note_hashes[i] == N {
            propagated_note_hashes.push(note_hashes[i]);
        }
    }

    let mut propagated_nullifiers = BoundedVec::new();
    for i in 0..N {
        if transient_note_hash_indexes_for_nullifiers[i] == M {
            propagated_nullifiers.push(nullifiers[i]);
        }
    }

    let mut propagated_logs = BoundedVec::new();
    for i in 0..logs.len() {
        let log = logs[i];
        let linked_note_hash_propagated = propagated_note_hashes.storage.any(|n: ScopedNoteHash| (n.counter() == log.note_hash_counter));
        if linked_note_hash_propagated {
            propagated_logs.push(log);
        }
    }

    (propagated_note_hashes.storage, propagated_nullifiers.storage, propagated_logs.storage)
}
