use dep::aztec::state_vars::{
    immutable_singleton::ImmutableSingleton
};

use crate::ecdsa_public_key_note::{
    EcdsaPublicKeyNote,
    EcdsaPublicKeyNoteInterface,
    ECDSA_PUBLIC_KEY_NOTE_LEN,
};

struct Storage {
    public_key: ImmutableSingleton<EcdsaPublicKeyNote, ECDSA_PUBLIC_KEY_NOTE_LEN>,
}

impl Storage {
    fn init() -> Self {
        Storage {
            public_key: ImmutableSingleton::new(1, EcdsaPublicKeyNoteInterface)
        }
    }
}