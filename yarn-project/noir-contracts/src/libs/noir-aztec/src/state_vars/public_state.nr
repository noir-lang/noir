use crate::oracle::storage::storage_read;
use crate::oracle::storage::storage_write;
use crate::state_vars::public_state_value::PublicStateValueInterface;

struct PublicState<T, N> {
    storage_slot: Field,
    value_interface: PublicStateValueInterface<T, N>,
}

impl<T, N> PublicState<T, N> {
    fn new(storage_slot: Field, value_interface: PublicStateValueInterface<T, N>) -> Self {
        PublicState { storage_slot, value_interface }
    }

    fn read(self) -> T {
        storage_read(self.storage_slot, self.value_interface.deserialise)
    }

    fn write(self, value: T) {
        let serialise = self.value_interface.serialise;
        let fields = serialise(value);
        storage_write(self.storage_slot, fields);
    }
}