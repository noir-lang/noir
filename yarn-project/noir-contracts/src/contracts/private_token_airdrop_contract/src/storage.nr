use crate::claim_note::{ClaimNote, ClaimNoteMethods, CLAIM_NOTE_LEN};
use dep::aztec::context::{PrivateContext, PublicContext};
use dep::aztec::state_vars::{map::Map, set::Set};
use dep::std::option::Option;
use dep::value_note::value_note::{ValueNote, ValueNoteMethods, VALUE_NOTE_LEN};

struct Storage {
    balances: Map<Set<ValueNote, VALUE_NOTE_LEN>>,
    claims: Set<ClaimNote, CLAIM_NOTE_LEN>,
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
            claims: Set::new(private_context, public_context, 2, ClaimNoteMethods),
        }
    }
}
