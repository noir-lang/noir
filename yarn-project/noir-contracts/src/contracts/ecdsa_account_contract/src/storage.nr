use dep::aztec::state_vars::immutable_singleton::ImmutableSingleton;

use crate::ecdsa_public_key_note::{
    EcdsaPublicKeyNote, EcdsaPublicKeyNoteInterface, ECDSA_PUBLIC_KEY_NOTE_LEN,
};
use dep::aztec::context::{PrivateContext, PublicContext};
use dep::std::option::Option;

struct Storage {
    public_key: ImmutableSingleton<EcdsaPublicKeyNote, ECDSA_PUBLIC_KEY_NOTE_LEN>,
}

impl Storage {
    fn init(private_context: Option<&mut PrivateContext>, _: Option<&mut PublicContext>) -> Self {
        Storage {
            public_key: ImmutableSingleton::new(private_context, 1, EcdsaPublicKeyNoteInterface),
        }
    }
}
