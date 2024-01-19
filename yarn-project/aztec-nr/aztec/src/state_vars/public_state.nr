use crate::context::{Context};
use crate::oracle::storage::storage_read;
use crate::oracle::storage::storage_write;
use crate::types::type_serialization::TypeSerializationInterface;
use dep::std::option::Option;

// docs:start:public_state_struct
struct PublicState<T, T_SERIALIZED_LEN> {
    context: Context,
    storage_slot: Field,
    serialization_methods: TypeSerializationInterface<T, T_SERIALIZED_LEN>,
}
// docs:end:public_state_struct

impl<T, T_SERIALIZED_LEN> PublicState<T, T_SERIALIZED_LEN> {
    // docs:start:public_state_struct_new
    pub fn new(
        // Note: Passing the contexts to new(...) just to have an interface compatible with a Map.
        context: Context,
        storage_slot: Field,
        serialization_methods: TypeSerializationInterface<T, T_SERIALIZED_LEN>,
    ) -> Self {
        assert(storage_slot != 0, "Storage slot 0 not allowed. Storage slots must start from 1.");
        PublicState {
            context,
            storage_slot,
            serialization_methods,
        }
    }
    // docs:end:public_state_struct_new

    // docs:start:public_state_struct_read
    pub fn read(self) -> T {
        assert(self.context.private.is_none(), "Public state reads only supported in public functions");
        storage_read(self.storage_slot, self.serialization_methods.deserialize)
    }
    // docs:end:public_state_struct_read

    // docs:start:public_state_struct_write
    pub fn write(self, value: T) {
        assert(self.context.private.is_none(), "Public state writes only supported in public functions");
        let serialize = self.serialization_methods.serialize;
        let fields = serialize(value);
        storage_write(self.storage_slot, fields);
    }
    // docs:end:public_state_struct_write
}
