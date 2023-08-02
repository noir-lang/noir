use dep::aztec::state_vars::{
    set::Set
};

use crate::address_note::{
    AddressNote,
    AddressNoteMethods,
    ADDRESS_NOTE_LEN,
};

struct Storage {
    owners: Set<AddressNote, ADDRESS_NOTE_LEN>,
}

impl Storage {
    fn init() -> Self {
        Storage {
            owners: Set::new(1, AddressNoteMethods),
        }
    }
}