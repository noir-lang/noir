use dep::types::abis::{note_hash::NoteHashContext, nullifier::Nullifier};

pub fn squash_transient_note_hashes<N>(note_hashes: [NoteHashContext; N]) -> [NoteHashContext; N] {
    let mut final_note_hashes = [NoteHashContext::empty(); N];

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

pub fn squash_transient_nullifiers<N>(nullifiers: [Nullifier; N]) -> [Nullifier; N] {
    let mut final_nullifiers = [Nullifier::empty(); N];

    let mut num_nullifiers = 0;
    for i in 0..N {
        let nullifier = nullifiers[i];
        if nullifier.note_hash == 0 {
            final_nullifiers[num_nullifiers] = nullifier;
            num_nullifiers += 1;
        }
    }

    final_nullifiers
}
