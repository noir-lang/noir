use dep::aztec::state_vars::{
    map::Map,
    set::Set
};
use dep::token_utils::value_note::{
    value_note::ValueNote,
    value_note::ValueNoteInterface,
    value_note::VALUE_NOTE_LEN,
};
use dep::custom_notes::claim_note::{CLAIM_NOTE_LEN, ClaimNote, ClaimNoteInterface};

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