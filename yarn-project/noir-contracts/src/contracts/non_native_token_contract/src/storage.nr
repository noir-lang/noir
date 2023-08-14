mod transparent_note;

use crate::transparent_note::{
    TransparentNote,
    TransparentNoteMethods,
    TRANSPARENT_NOTE_LEN,
};

use dep::value_note::value_note::{
    ValueNote,
    ValueNoteMethods,
    VALUE_NOTE_LEN,
};

use dep::aztec::{
    state_vars::{
        map::Map,
        set::Set,
        public_state::PublicState,
    },
    types::type_serialisation::field_serialisation::{
        FIELD_SERIALISED_LEN,
        FieldSerialisationMethods,
    },
};

struct Storage {
    balances: Map<Set<ValueNote, VALUE_NOTE_LEN>>,

    pending_shields: Set<TransparentNote, TRANSPARENT_NOTE_LEN>,

    public_balances: Map<PublicState<Field, FIELD_SERIALISED_LEN>>,
}

impl Storage {
    fn init() -> Self {
        Storage {
            balances: Map::new(1, |s| Set::new(s, ValueNoteMethods)),
            pending_shields: Set::new(2, TransparentNoteMethods),
            public_balances: Map::new(3, |s| PublicState::new(s, FieldSerialisationMethods)),
        }
    }
}