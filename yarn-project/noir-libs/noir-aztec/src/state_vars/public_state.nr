use crate::oracle::storage::storage_read;
use crate::oracle::storage::storage_write;
use crate::types::type_serialisation::TypeSerialisationInterface;

struct PublicState<T, T_SERIALISED_LEN> {
    storage_slot: Field,
    serialisation_methods: TypeSerialisationInterface<T, T_SERIALISED_LEN>,
}

impl<T, T_SERIALISED_LEN> PublicState<T, T_SERIALISED_LEN> {
    fn new(storage_slot: Field, serialisation_methods: TypeSerialisationInterface<T, T_SERIALISED_LEN>) -> Self {
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