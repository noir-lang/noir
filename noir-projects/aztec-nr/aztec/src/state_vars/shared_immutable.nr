use crate::{
    context::Context, history::public_value_inclusion::prove_public_value_inclusion,
    oracle::{storage::{storage_read, storage_write}}, state_vars::storage::Storage
};
use dep::protocol_types::{constants::INITIALIZATION_SLOT_SEPARATOR, traits::{Deserialize, Serialize}};

// Just like PublicImmutable but with the ability to read from private functions.
struct SharedImmutable<T>{
    context: Context,
    storage_slot: Field,
}

impl<T> Storage<T> for SharedImmutable<T> {}

impl<T> SharedImmutable<T> {
    pub fn new(
        // Note: Passing the contexts to new(...) just to have an interface compatible with a Map.
        context: Context,
        storage_slot: Field
    ) -> Self {
        assert(storage_slot != 0, "Storage slot 0 not allowed. Storage slots must start from 1.");
        Self { context, storage_slot }
    }

    // Intended to be only called once. 
    pub fn initialize<T_SERIALIZED_LEN>(self, value: T) where T: Serialize<T_SERIALIZED_LEN> {
        assert(
            self.context.private.is_none(), "SharedImmutable can only be initialized from public functions"
        );
        // TODO(#4738): Uncomment the following assert
        // assert(
        //     self.context.public.unwrap_unchecked().is_deployment(), "SharedImmutable can only be initialized during contract deployment"
        // );

        // We check that the struct is not yet initialized by checking if the initialization slot is 0
        let initialization_slot = INITIALIZATION_SLOT_SEPARATOR + self.storage_slot;
        let fields_read: [Field; 1] = storage_read(initialization_slot);
        assert(fields_read[0] == 0, "SharedImmutable already initialized");

        // We populate the initialization slot with a non-zero value to indicate that the struct is initialized
        storage_write(initialization_slot, [0xdead]);

        let fields_write = T::serialize(value);
        storage_write(self.storage_slot, fields_write);
    }

    pub fn read_public<T_SERIALIZED_LEN>(self) -> T where T: Deserialize<T_SERIALIZED_LEN> {
        assert(self.context.private.is_none(), "Public read only supported in public functions");
        let fields = storage_read(self.storage_slot);
        T::deserialize(fields)
    }

    pub fn read_private<T_SERIALIZED_LEN>(self) -> T  where T: Deserialize<T_SERIALIZED_LEN> {
        assert(self.context.public.is_none(), "Private read only supported in private functions");
        let private_context = self.context.private.unwrap();

        // Read the value from storage (using the public tree)
        let fields = storage_read(self.storage_slot);

        // Loop over the fields and prove their inclusion in the public tree
        for i in 0..fields.len() {
            // TODO: Update membership proofs to use current header (Requires #4179)
            // Currently executing unnecessary computation:
            // - a membership proof of the value in the public tree of the header 
            prove_public_value_inclusion(
                fields[i],
                self.storage_slot + i as Field,
                (*private_context).this_address(),
                (*private_context)
            )
        }
        T::deserialize(fields)
    }
}
