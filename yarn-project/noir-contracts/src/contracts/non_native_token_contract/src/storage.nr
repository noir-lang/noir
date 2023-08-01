use dep::aztec::state_vars::{
    map::Map,
    set::Set,
    public_state::PublicState,
    type_serialisation_interface::field_serialisation_interface::{
        FIELD_SERIALISED_LEN,
        FieldSerialisationInterface,
    },
};

use dep::value_note::value_note::{
    ValueNote,
    ValueNoteInterface,
    VALUE_NOTE_LEN,
};

struct Storage {
    balances: Map<Set<ValueNote, VALUE_NOTE_LEN>>,

    public_balances: Map<PublicState<Field, FIELD_SERIALISED_LEN>>,
}

impl Storage {
    fn init() -> Self {
        Storage {
            balances: Map::new(1, |slot| Set::new(slot, ValueNoteInterface)),
            public_balances: Map::new(2, |slot| PublicState::new(slot, FieldSerialisationInterface)),
        }
    }
}