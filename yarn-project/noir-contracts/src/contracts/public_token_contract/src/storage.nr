use dep::aztec::state_vars::map::Map;
use dep::aztec::state_vars::public_state::PublicState;
use dep::aztec::state_vars::type_serialisation_interface::field_serialisation_interface::FieldSerialisationInterface;
use dep::aztec::state_vars::type_serialisation_interface::field_serialisation_interface::FIELD_SERIALISED_LEN;

struct Storage {
    balances: Map<PublicState<Field, FIELD_SERIALISED_LEN>>,
}


impl Storage {
    fn init() -> Self {
        Storage {
            balances: Map::new(1, |slot| PublicState::new(slot, FieldSerialisationInterface)),
        }
    }
}