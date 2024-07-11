use crate::context::{PublicContext, UnconstrainedContext};
use crate::oracle::storage::storage_read;
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

impl<T, T_SERIALIZED_LEN> PublicMutable<T, &mut PublicContext> where T: Serialize<T_SERIALIZED_LEN> + Deserialize<T_SERIALIZED_LEN> {
    // docs:start:public_mutable_struct_read
    pub fn read(self) -> T {
        self.context.storage_read(self.storage_slot)
    }
    // docs:end:public_mutable_struct_read

    // docs:start:public_mutable_struct_write
    pub fn write(self, value: T) {
        self.context.storage_write(self.storage_slot, value);
    }
    // docs:end:public_mutable_struct_write
}

impl<T, T_SERIALIZED_LEN> PublicMutable<T, UnconstrainedContext> where T: Deserialize<T_SERIALIZED_LEN> {
    unconstrained pub fn read(self) -> T {
        self.context.storage_read(self.storage_slot)
    }
}
