use crate::{context::Context, oracle::{storage::{storage_read, storage_write}}, state_vars::storage::Storage};
use dep::protocol_types::{constants::INITIALIZATION_SLOT_SEPARATOR, traits::{Deserialize, Serialize}};

// Just like SharedImmutable but without the ability to read from private functions.
// docs:start:public_immutable_struct
struct PublicImmutable<T> {
    context: Context,
    storage_slot: Field,
}
// docs:end:public_immutable_struct

impl<T> Storage<T> for PublicImmutable<T> {}

impl<T> PublicImmutable<T> {
    // docs:start:public_immutable_struct_new
    pub fn new(
        // Note: Passing the contexts to new(...) just to have an interface compatible with a Map.
        context: Context,
        storage_slot: Field
    ) -> Self {
        assert(storage_slot != 0, "Storage slot 0 not allowed. Storage slots must start from 1.");
        PublicImmutable { context, storage_slot }
    }
    // docs:end:public_immutable_struct_new

    // docs:start:public_immutable_struct_write
    pub fn initialize<T_SERIALIZED_LEN>(self, value: T) where T: Serialize<T_SERIALIZED_LEN> {
        assert(
            self.context.private.is_none(), "PublicImmutable can only be initialized from public functions"
        );
        // TODO(#4738): Uncomment the following assert
        // assert(
        //     self.context.public.unwrap_unchecked().is_deployment(), "PublicImmutable can only be initialized during contract deployment"
        // );

        // We check that the struct is not yet initialized by checking if the initialization slot is 0
        let initialization_slot = INITIALIZATION_SLOT_SEPARATOR + self.storage_slot;
        let fields_read: [Field; 1] = storage_read(initialization_slot);
        assert(fields_read[0] == 0, "PublicImmutable already initialized");

        // We populate the initialization slot with a non-zero value to indicate that the struct is initialized
        storage_write(initialization_slot, [0xdead]);

        let fields_write = T::serialize(value);
        storage_write(self.storage_slot, fields_write);
    }
    // docs:end:public_immutable_struct_write

    // docs:start:public_immutable_struct_read
    pub fn read<T_SERIALIZED_LEN>(self) -> T where T: Deserialize<T_SERIALIZED_LEN> {
        assert(self.context.private.is_none(), "PublicImmutable reads only supported in public functions");
        let fields = storage_read(self.storage_slot);
        T::deserialize(fields)
    }
    // docs:end:public_immutable_struct_read
}
