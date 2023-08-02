use dep::aztec::state_vars::{
    immutable_singleton::ImmutableSingleton,
    map::Map,
    set::Set
};
use crate::address_note::{
    AddressNote,
    AddressNoteMethods,
    ADDRESS_NOTE_LEN,
};
use dep::value_note::value_note::{
    ValueNote,
    ValueNoteMethods,
    VALUE_NOTE_LEN,
};

struct Storage {
    sender: ImmutableSingleton<AddressNote, ADDRESS_NOTE_LEN>,
    recipient: ImmutableSingleton<AddressNote, ADDRESS_NOTE_LEN>,
    balances: Map<Set<ValueNote, VALUE_NOTE_LEN>>,
}

impl Storage {
    fn init() -> Self {
        Storage {
            sender: ImmutableSingleton::new(1, AddressNoteMethods),
            recipient: ImmutableSingleton::new(2, AddressNoteMethods),
            balances: Map::new(3, |slot| Set::new(slot, ValueNoteMethods)),
        }
    }
}