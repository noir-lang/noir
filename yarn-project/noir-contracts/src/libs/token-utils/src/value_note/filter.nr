use dep::aztec::constants_gen::READ_REQUESTS_LENGTH;
use crate::value_note::value_note::ValueNote;

fn get_2_notes<P>(notes: [ValueNote; READ_REQUESTS_LENGTH], _x: P) -> [ValueNote; 2] {
    [notes[0], notes[1]]
}

fn get_all_notes<P>(notes: [ValueNote; READ_REQUESTS_LENGTH], _x: P) -> [ValueNote; READ_REQUESTS_LENGTH] {
    notes
}