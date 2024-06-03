use crate::context::{PublicContext, UnconstrainedContext};
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

impl<T> PublicMutable<T, UnconstrainedContext> {
    pub fn read<T_SERIALIZED_LEN>(self) -> T where T: Deserialize<T_SERIALIZED_LEN> {
        // This looks the same as the &mut PublicContext impl, but is actually very different. In public execution the
        // storage read oracle gets transpiled to SLOAD opcodes, whereas in unconstrained execution the PXE returns
        // historical data.
        let fields = storage_read(self.storage_slot);
        T::deserialize(fields)
    }
}
