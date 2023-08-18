use dep::std::option::Option;
use dep::aztec::constants_gen::MAX_READ_REQUESTS_PER_CALL;
use crate::value_note::ValueNote;

fn get_1_note<P>(notes: [Option<ValueNote>; MAX_READ_REQUESTS_PER_CALL], _x: P) -> [Option<ValueNote>; 1] {
    [notes[0]]
}

fn get_max_notes<P>(notes: [Option<ValueNote>; MAX_READ_REQUESTS_PER_CALL], _x: P) -> [Option<ValueNote>; MAX_READ_REQUESTS_PER_CALL] {
    notes
}

fn filter_notes_min_sum(notes: [Option<ValueNote>; MAX_READ_REQUESTS_PER_CALL], min_sum: Field) -> [Option<ValueNote>; MAX_READ_REQUESTS_PER_CALL] {
    let mut selected = [Option::none(); MAX_READ_REQUESTS_PER_CALL];
    let mut sum = 0;
    for i in 0..notes.len() {
        if notes[i].is_some() & (sum < min_sum as u120) {
            let note = notes[i].unwrap_unchecked();
            selected[i] = Option::some(note);
            sum += note.value as u120;
        }
    }
    selected
}