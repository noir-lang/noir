use crate::address_note::{AddressNote, AddressNoteMethods, ADDRESS_NOTE_LEN};
use dep::aztec::context::{PrivateContext, PublicContext};
use dep::aztec::state_vars::{immutable_singleton::ImmutableSingleton, map::Map, set::Set};
use dep::std::option::Option;
use dep::value_note::value_note::{ValueNote, ValueNoteMethods, VALUE_NOTE_LEN};

struct Storage {
    sender: ImmutableSingleton<AddressNote, ADDRESS_NOTE_LEN>,
    recipient: ImmutableSingleton<AddressNote, ADDRESS_NOTE_LEN>,
    balances: Map<Set<ValueNote, VALUE_NOTE_LEN>>,
}

impl Storage {
    fn init(
        private_context: Option<&mut PrivateContext>,
        public_context: Option<&mut PublicContext>,
    ) -> Self {
        Storage {
            sender: ImmutableSingleton::new(private_context, 1, AddressNoteMethods),
            recipient: ImmutableSingleton::new(private_context, 2, AddressNoteMethods),
            balances: Map::new(
                private_context,
                public_context,
                3,
                |private_context, public_context, slot| {
                    Set::new(private_context, public_context, slot, ValueNoteMethods)
                },
            ),
        }
    }
}
