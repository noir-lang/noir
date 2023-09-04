mod transparent_note;

use crate::transparent_note::{TransparentNote, TransparentNoteMethods, TRANSPARENT_NOTE_LEN};

use dep::aztec::context::{PrivateContext, PublicContext};
use dep::aztec::{
    state_vars::{map::Map, public_state::PublicState, set::Set},
    types::type_serialisation::field_serialisation::{
        FieldSerialisationMethods, FIELD_SERIALISED_LEN,
    },
};
use dep::std::option::Option;
use dep::value_note::value_note::{ValueNote, ValueNoteMethods, VALUE_NOTE_LEN};

struct Storage {
    balances: Map<Set<ValueNote, VALUE_NOTE_LEN>>,

    pending_shields: Set<TransparentNote, TRANSPARENT_NOTE_LEN>,

    public_balances: Map<PublicState<Field, FIELD_SERIALISED_LEN>>,
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
                1, // Storage slot
                |private_context, public_context, slot| {
                    Set::new(private_context, public_context, slot, ValueNoteMethods)
                },
            ),
            pending_shields: Set::new(private_context, public_context, 2, TransparentNoteMethods),
            public_balances: Map::new(
                private_context,
                public_context,
                3,
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
