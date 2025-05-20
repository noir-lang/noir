use dep::reset_kernel_lib::TransientDataIndexHint;
use dep::types::abis::{
    note_hash::ScopedNoteHash,
    nullifier::ScopedNullifier,
    private_log::PrivateLogData,
    side_effect::{Ordered, scoped::Scoped},
};

pub unconstrained fn squash_transient_data<let M: u32, let N: u32, let P: u32, let NUM_TRANSIENT_DATA_INDEX_HINTS: u32>(
    note_hashes: [ScopedNoteHash; M],
    nullifiers: [ScopedNullifier; N],
    logs: [Scoped<PrivateLogData>; P],
    transient_data_index_hints: [TransientDataIndexHint; NUM_TRANSIENT_DATA_INDEX_HINTS],
) -> ([ScopedNoteHash; M], [ScopedNullifier; N], [Scoped<PrivateLogData>; P]) {
    let mut transient_nullifier_indexes_for_note_hashes = [N; M];
    let mut transient_note_hash_indexes_for_nullifiers = [M; N];
    for i in 0..transient_data_index_hints.len() {
        let hint = transient_data_index_hints[i];
        if hint.note_hash_index != M {
            transient_nullifier_indexes_for_note_hashes[hint.note_hash_index] =
                hint.nullifier_index;
            transient_note_hash_indexes_for_nullifiers[hint.nullifier_index] = hint.note_hash_index;
        }
    }

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
        if log.inner.note_hash_counter == 0 {
            propagated_logs.push(log);
        } else {
            let linked_note_hash_propagated = propagated_note_hashes
                .storage()
                .any(|n: ScopedNoteHash| (n.counter() == log.inner.note_hash_counter));
            if linked_note_hash_propagated {
                propagated_logs.push(log);
            }
        }
    }

    (propagated_note_hashes.storage(), propagated_nullifiers.storage(), propagated_logs.storage())
}
