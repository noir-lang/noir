use crate::oracle::storage::storage_read;
use crate::oracle::storage::storage_write;
use crate::state_vars::type_serialisation::TypeSerialisationInterface;

struct PublicState<T, N> {
    storage_slot: Field,
    serialisation_methods: TypeSerialisationInterface<T, N>,
}

impl<T, N> PublicState<T, N> {
    fn new(storage_slot: Field, serialisation_methods: TypeSerialisationInterface<T, N>) -> Self {
        PublicState { storage_slot, serialisation_methods }
    }

    fn read(self) -> T {
        storage_read(self.storage_slot, self.serialisation_methods.deserialise)
    }

    fn write(self, value: T) {
        let serialise = self.serialisation_methods.serialise;
        let fields = serialise(value);
        storage_write(self.storage_slot, fields);
    }
}