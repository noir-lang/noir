use dep::aztec::context::{PrivateContext, PublicContext};
use dep::aztec::state_vars::{
    map::Map,
    // highlight-start:PublicState
    public_state::PublicState,
    // highlight-end:PublicState
};
use dep::std::option::Option;

use dep::aztec::types::type_serialisation::field_serialisation::{
    FieldSerialisationMethods, FIELD_SERIALISED_LEN,
};

struct Storage {
    balances: Map<PublicState<Field, FIELD_SERIALISED_LEN>>,
}

impl Storage {
    fn init(
        private_context: Option<&mut PrivateContext>,
        public_context: Option<&mut PublicContext>,
    ) -> Self {
        Storage {
            balances: Map::new(
                private_context,
                public_context,
                1,
                |private_context, public_context, slot| {
                    PublicState::new(
                        private_context,
                        public_context,
                        slot,
                        FieldSerialisationMethods,
                    )
                },
            ),
        }
    }
}
