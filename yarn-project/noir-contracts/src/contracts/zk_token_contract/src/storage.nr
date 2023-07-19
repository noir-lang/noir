use dep::aztec::state_vars::{
    map::Map,
    set::Set
};
use dep::value_note::value_note::{
    ValueNote,
    ValueNoteInterface,
    VALUE_NOTE_LEN,
};
use crate::claim_note::{CLAIM_NOTE_LEN, ClaimNote, ClaimNoteInterface};

struct Storage {
    balances: Map<Set<ValueNote, VALUE_NOTE_LEN>>,
    claims: Set<ClaimNote, CLAIM_NOTE_LEN>,
}

impl Storage {
    fn init() -> Self {
        Storage {
            balances: Map::new(1, |s| Set::new(s, ValueNoteInterface)),
            claims: Set::new(2, ClaimNoteInterface),
        }
    }
}