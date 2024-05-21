use crate::context::PublicContext;
use crate::oracle::storage::storage_read;
use crate::oracle::storage::storage_write;
use dep::protocol_types::traits::{Deserialize, Serialize};
use crate::state_vars::storage::Storage;

// docs:start:public_mutable_struct
struct PublicMutable<T, Context> {
    context: Context,
    storage_slot: Field,
}
// docs:end:public_mutable_struct

impl<T, Context> Storage<T> for PublicMutable<T, Context> {}

impl<T, Context> PublicMutable<T, Context> {
    // docs:start:public_mutable_struct_new
    pub fn new(
        // Note: Passing the contexts to new(...) just to have an interface compatible with a Map.
        context: Context,
        storage_slot: Field
    ) -> Self {
        assert(storage_slot != 0, "Storage slot 0 not allowed. Storage slots must start from 1.");
        PublicMutable { context, storage_slot }
    }
    // docs:end:public_mutable_struct_new
}

impl<T> PublicMutable<T, &mut PublicContext> {
    // docs:start:public_mutable_struct_read
    pub fn read<T_SERIALIZED_LEN>(self) -> T where T: Deserialize<T_SERIALIZED_LEN> {
        let fields = storage_read(self.storage_slot);
        T::deserialize(fields)
    }
    // docs:end:public_mutable_struct_read

    // docs:start:public_mutable_struct_write
    pub fn write<T_SERIALIZED_LEN>(self, value: T) where T: Serialize<T_SERIALIZED_LEN> {
        let fields = T::serialize(value);
        storage_write(self.storage_slot, fields);
    }
    // docs:end:public_mutable_struct_write
}

impl<T> PublicMutable<T, ()> {
    pub fn read<T_SERIALIZED_LEN>(self) -> T where T: Deserialize<T_SERIALIZED_LEN> {
        // Note that this is the exact same implementation as for public execution, though it might change in the future
        // since unconstrained execution might not rely on the same oracles as used for public execution (which 
        // transpile to AVM opcodes).
        let fields = storage_read(self.storage_slot);
        T::deserialize(fields)
    }
}
