use dep::types::{abis::{note_hash::ScopedNoteHash, log_hash::NoteLogHash}};

unconstrained pub fn get_transient_or_propagated_note_hash_indexes_for_logs<NUM_LOGS, NUM_NOTE_HASHES>(
    note_logs: [NoteLogHash; NUM_LOGS],
    note_hashes: [ScopedNoteHash; NUM_NOTE_HASHES],
    expected_note_hashes: [ScopedNoteHash; NUM_NOTE_HASHES]
) -> [u32; NUM_LOGS] {
    let mut indexes = [0; NUM_LOGS];
    for i in 0..note_logs.len() {
        let log_note_hash_counter = note_logs[i].note_hash_counter;
        let mut propagated = false;
        for j in 0..expected_note_hashes.len() {
            if !propagated & (expected_note_hashes[j].counter() == log_note_hash_counter) {
                indexes[i] = j;
                propagated = true;
            }
        }
        if !propagated {
            for j in 0..note_hashes.len() {
                if note_hashes[j].counter() == log_note_hash_counter {
                    indexes[i] = j;
                }
            }
        }
    }

    indexes
}
