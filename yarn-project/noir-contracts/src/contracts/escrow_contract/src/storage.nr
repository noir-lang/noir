use dep::aztec::context::{PrivateContext, PublicContext};
use dep::aztec::state_vars::set::Set;
use dep::std::option::Option;

use crate::address_note::{AddressNote, AddressNoteMethods, ADDRESS_NOTE_LEN};

struct Storage {
    owners: Set<AddressNote, ADDRESS_NOTE_LEN>,
}

impl Storage {
    fn init(
        private_context: Option<&mut PrivateContext>,
        public_context: Option<&mut PublicContext>,
    ) -> Self {
        Storage {
            owners: Set::new(private_context, public_context, 1, AddressNoteMethods),
        }
    }
}
