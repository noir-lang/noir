use crate::context::{Context};
use crate::oracle::storage::storage_read;
use crate::oracle::storage::storage_write;
use crate::types::type_serialization::TypeSerializationInterface;
use dep::std::option::Option;

struct PublicState<T, T_SERIALIZED_LEN> {
    storage_slot: Field,
    serialization_methods: TypeSerializationInterface<T, T_SERIALIZED_LEN>,
}

impl<T, T_SERIALIZED_LEN> PublicState<T, T_SERIALIZED_LEN> {
    fn new(
        // Note: Passing the contexts to new(...) just to have an interface compatible with a Map.
        _: Context,
        storage_slot: Field,
        serialization_methods: TypeSerializationInterface<T, T_SERIALIZED_LEN>,
    ) -> Self {
        assert(storage_slot != 0, "Storage slot 0 not allowed. Storage slots must start from 1.");
        PublicState {
            storage_slot,
            serialization_methods,
        }
    }

    fn read(self) -> T {
        storage_read(self.storage_slot, self.serialization_methods.deserialize)
    }

    fn write(self, value: T) {
        let serialize = self.serialization_methods.serialize;
        let fields = serialize(value);
        storage_write(self.storage_slot, fields);
    }
}
