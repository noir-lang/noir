use dep::aztec::state_vars::public_state::PublicState;
use dep::aztec::state_vars::public_state_value::field_value::FieldValueInterface;
use dep::aztec::state_vars::public_state_value::field_value::FIELD_VALUE_LEN;

struct Storage {
    current_value: PublicState<Field, FIELD_VALUE_LEN>,
}

impl Storage {
    fn init() -> Self {
        Storage {
            current_value: PublicState::new(1, FieldValueInterface),
        }
    }
}