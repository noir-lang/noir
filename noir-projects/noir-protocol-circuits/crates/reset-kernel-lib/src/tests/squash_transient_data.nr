use dep::types::abis::{note_hash::ScopedNoteHash, nullifier::ScopedNullifier, log_hash::NoteLogHash};

pub fn squash_transient_note_hashes<N>(note_hashes: [ScopedNoteHash; N]) -> [ScopedNoteHash; N] {
    let mut final_note_hashes = [ScopedNoteHash::empty(); N];

    let mut num_note_hashes = 0;
    for i in 0..N {
        let note_hash = note_hashes[i];
        if note_hash.nullifier_counter == 0 {
            final_note_hashes[num_note_hashes] = note_hash;
            num_note_hashes += 1;
        }
    }

    final_note_hashes
}

pub fn squash_transient_nullifiers<N>(nullifiers: [ScopedNullifier; N]) -> [ScopedNullifier; N] {
    let mut final_nullifiers = [ScopedNullifier::empty(); N];

    let mut num_nullifiers = 0;
    for i in 0..N {
        let nullifier = nullifiers[i];
        if nullifier.nullified_note_hash() == 0 {
            final_nullifiers[num_nullifiers] = nullifier;
            num_nullifiers += 1;
        }
    }

    final_nullifiers
}

pub fn squash_transient_logs<N, M>(
    logs: [NoteLogHash; N],
    note_hashes: [ScopedNoteHash; M]
) -> [NoteLogHash; N] {
    let mut final_logs = [NoteLogHash::empty(); N];

    let mut num_logs = 0;
    for i in 0..N {
        let mut log = logs[i];
        let is_nullified = note_hashes.any(|n: ScopedNoteHash| (n.counter() == log.note_hash_counter) & (n.nullifier_counter != 0));
        if !is_nullified {
            final_logs[num_logs] = log;
            num_logs += 1;
        }
    }

    final_logs
}
