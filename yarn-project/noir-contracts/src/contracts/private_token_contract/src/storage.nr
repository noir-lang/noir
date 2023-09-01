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
    // maps an aztec address to its balance
    balances: Map<Set<ValueNote, VALUE_NOTE_LEN>>,
}

impl Storage {
    fn init() -> Self {
        Storage {
            balances: Map::new(1, |slot| Set::new(slot, ValueNoteMethods)),
        }
    }
}
