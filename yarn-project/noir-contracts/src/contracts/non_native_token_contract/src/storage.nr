use dep::aztec::state_vars::{
    map::Map,
    set::Set,
    public_state::PublicState,
    type_serialisation::field_serialisation::{
        FIELD_SERIALISED_LEN,
        FieldSerialisationMethods,
    },
};

use dep::value_note::value_note::{
    ValueNote,
    ValueNoteMethods,
    VALUE_NOTE_LEN,
};

struct Storage {
    balances: Map<Set<ValueNote, VALUE_NOTE_LEN>>,

    public_balances: Map<PublicState<Field, FIELD_SERIALISED_LEN>>,
}

impl Storage {
    fn init() -> Self {
        Storage {
            balances: Map::new(1, |slot| Set::new(slot, ValueNoteMethods)),
            public_balances: Map::new(2, |slot| PublicState::new(slot, FieldSerialisationMethods)),
        }
    }
}