use dep::aztec::state_vars::{
    map::Map,
    set::Set
};
use dep::value_note::value_note::{
    ValueNote,
    ValueNoteMethods,
    VALUE_NOTE_LEN,
};

struct Storage {
    balances: Map<Set<ValueNote, VALUE_NOTE_LEN>>,
}

impl Storage {
    fn init() -> Self {
        Storage {
            balances: Map::new(1, |s| Set::new(s, ValueNoteMethods)),
        }
    }
}