use dep::aztec::state_vars::map::Map;
use dep::aztec::state_vars::set::Set;
use crate::notes::value_note::ValueNote;
use crate::notes::value_note::ValueNoteInterface;
use crate::notes::value_note::VALUE_NOTE_LEN;

struct Storage {
    balances: Map<Set<ValueNote, VALUE_NOTE_LEN>>,
}

impl Storage {
    fn init() -> Self {
        Storage {
            balances: Map::new(1, |s| Set::new(s, ValueNoteInterface)),
        }
    }
}