// docs:start:PublicState
use dep::aztec::state_vars::{
    // highlight-start:PublicState
    public_state::PublicState,  
    type_serialisation::field_serialisation::{
        FieldSerialisationMethods,
        FIELD_SERIALISED_LEN,
    },
    // highlight-end:PublicState
};

struct Storage {
    // highlight-next-line:PublicState
    a: PublicState<Field, FIELD_SERIALISED_LEN>,
}

impl Storage {
    fn init() -> Self {
        Storage {
            // highlight-next-line:PublicState
            a: PublicState::new(1, FieldSerialisationMethods),
        }
    }
}
// docs:end:PublicState