use dep::aztec::constants_gen::MAX_READ_REQUESTS_PER_CALL;
use crate::value_note::ValueNote;
use dep::aztec::types::option::Option;

fn get_2_notes<P>(notes: [Option<ValueNote>; MAX_READ_REQUESTS_PER_CALL], _x: P) -> [Option<ValueNote>; 2] {
    [notes[0], notes[1]]
}

fn get_4_notes<P>(notes: [Option<ValueNote>; MAX_READ_REQUESTS_PER_CALL], _x: P) -> [Option<ValueNote>; 4] {
    [notes[0], notes[1], notes[2], notes[3]]
}

fn get_all_notes<P>(notes: [Option<ValueNote>; MAX_READ_REQUESTS_PER_CALL], _x: P) -> [Option<ValueNote>; MAX_READ_REQUESTS_PER_CALL] {
    notes
}