use dep::aztec::state_vars::public_state::PublicState;
use dep::aztec::types::type_serialisation::field_serialisation::FieldSerialisationMethods;
use dep::aztec::types::type_serialisation::field_serialisation::FIELD_SERIALISED_LEN;

struct Storage {
    current_value: PublicState<Field, FIELD_SERIALISED_LEN>,
}

impl Storage {
    fn init() -> Self {
        Storage {
            current_value: PublicState::new(1, FieldSerialisationMethods),
        }
    }
}