use dep::aztec::context::{PrivateContext, PublicContext};
use dep::aztec::state_vars::public_state::PublicState;
use dep::aztec::types::type_serialisation::field_serialisation::FieldSerialisationMethods;
use dep::aztec::types::type_serialisation::field_serialisation::FIELD_SERIALISED_LEN;
use dep::std::option::Option;

struct Storage {
    current_value: PublicState<Field, FIELD_SERIALISED_LEN>,
}

impl Storage {
    fn init(
        private_context: Option<&mut PrivateContext>,
        public_context: Option<&mut PublicContext>,
    ) -> Self {
        Storage {
            current_value: PublicState::new(
                private_context,
                public_context,
                1,
                FieldSerialisationMethods,
            ),
        }
    }
}
