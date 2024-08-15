use dep::reset_kernel_lib::TransientDataIndexHint;
use dep::types::{abis::{log_hash::NoteLogHash, note_hash::ScopedNoteHash}, utils::arrays::find_index_hint};

unconstrained pub fn get_transient_or_propagated_note_hash_indexes_for_logs<let NUM_LOGS: u32, let NUM_NOTE_HASHES: u32, let NUM_INDEX_HINTS: u32>(
    note_logs: [NoteLogHash; NUM_LOGS],
    note_hashes: [ScopedNoteHash; NUM_NOTE_HASHES],
    expected_note_hashes: [ScopedNoteHash; NUM_NOTE_HASHES],
    transient_data_index_hints: [TransientDataIndexHint; NUM_INDEX_HINTS]
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
                    indexes[i] = find_index_hint(transient_data_index_hints, |hint: TransientDataIndexHint| hint.note_hash_index == j);
                }
            }
        }
    }

    indexes
}
