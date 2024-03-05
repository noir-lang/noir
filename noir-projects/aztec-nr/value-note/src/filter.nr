use dep::aztec::protocol_types::constants::MAX_NOTE_HASH_READ_REQUESTS_PER_CALL;
use crate::value_note::ValueNote;

pub fn filter_notes_min_sum(
    notes: [Option<ValueNote>; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL],
    min_sum: Field
) -> [Option<ValueNote>; MAX_NOTE_HASH_READ_REQUESTS_PER_CALL] {
    let mut selected = [Option::none(); MAX_NOTE_HASH_READ_REQUESTS_PER_CALL];
    let mut sum = U128::from_integer(0);
    for i in 0..notes.len() {
        if notes[i].is_some() & (sum < U128::from_integer(min_sum)) {
            let note = notes[i].unwrap_unchecked();
            selected[i] = Option::some(note);
            sum += U128::from_integer(note.value);
        }
    }
    selected
}
