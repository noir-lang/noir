use dep::value_note::value_note::{
    ValueNote,
    ValueNoteMethods,
    VALUE_NOTE_LEN,
};

use dep::non_native::{
    transparent_note::{
        TransparentNote,
        TransparentNoteMethods,
        TRANSPARENT_NOTE_LEN,
    },
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
    total_supply: PublicState<Field, FIELD_SERIALISED_LEN>,
    pending_shields: Set<TransparentNote, TRANSPARENT_NOTE_LEN>,
    public_balances: Map<PublicState<Field, FIELD_SERIALISED_LEN>>,
    public_allowances: Map<Map<PublicState<Field, FIELD_SERIALISED_LEN>>>,
}

impl Storage {
    fn init() -> Self {
        Storage {
            balances: Map::new(1, |s| Set::new(s, ValueNoteMethods)),
            total_supply: PublicState::new(2, FieldSerialisationMethods),
            pending_shields: Set::new(3, TransparentNoteMethods),
            public_balances: Map::new(4, |s| PublicState::new(s, FieldSerialisationMethods)),
            public_allowances: Map::new(5, |s1| Map::new(s1, |s2| PublicState::new(s2, FieldSerialisationMethods)))
        }
    }
}