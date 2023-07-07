use dep::aztec::abi::MAX_READ_REQUESTS;
use crate::value_note::value_note::ValueNote;

fn get_2_notes<P>(notes: [ValueNote; MAX_READ_REQUESTS], _x: P) -> [ValueNote; 2] {
    [notes[0], notes[1]]
}

fn get_all_notes<P>(notes: [ValueNote; MAX_READ_REQUESTS], _x: P) -> [ValueNote; MAX_READ_REQUESTS] {
    notes
}