use dep::aztec::state_vars::map::Map;
use dep::aztec::state_vars::set::Set;
use dep::aztec::state_vars::public_state::PublicState;
use dep::aztec::state_vars::public_state_value::field_value::FIELD_VALUE_LEN;
use dep::aztec::state_vars::public_state_value::field_value::FieldValueInterface;

use crate::value_note::value_note::ValueNote;
use crate::value_note::value_note::ValueNoteInterface;
use crate::value_note::value_note::VALUE_NOTE_LEN;

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