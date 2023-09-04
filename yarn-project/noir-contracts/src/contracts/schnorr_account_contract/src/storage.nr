use dep::aztec::context::{PrivateContext, PublicContext};
use dep::aztec::state_vars::immutable_singleton::ImmutableSingleton;
use dep::std::option::Option;

use crate::public_key_note::{PublicKeyNote, PublicKeyNoteMethods, PUBLIC_KEY_NOTE_LEN};

struct Storage {
    signing_public_key: ImmutableSingleton<PublicKeyNote, PUBLIC_KEY_NOTE_LEN>,
}

impl Storage {
    fn init(private_context: Option<&mut PrivateContext>, _: Option<&mut PublicContext>) -> Self {
        Storage {
            signing_public_key: ImmutableSingleton::new(private_context, 1, PublicKeyNoteMethods),
        }
    }
}
