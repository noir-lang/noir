use dep::aztec::state_vars::{
    map::Map,
    set::Set,
    public_state::PublicState,
    public_state_value::field_value::{
        FIELD_VALUE_LEN,
        FieldValueInterface,
    },
};

use dep::value_note::value_note::{
    ValueNote,
    ValueNoteInterface,
    VALUE_NOTE_LEN,
};

struct Storage {
    balances: Map<Set<ValueNote, VALUE_NOTE_LEN>>,

    public_balances: Map<PublicState<Field, FIELD_VALUE_LEN>>,
}

impl Storage {
    fn init() -> Self {
        Storage {
            balances: Map::new(1, |s| Set::new(s, ValueNoteInterface)),
            public_balances: Map::new(2, |s| PublicState::new(s, FieldValueInterface)),
        }
    }
}