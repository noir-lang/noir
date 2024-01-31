use crate::context::{Context};
use crate::oracle::{
    storage::{storage_read, storage_write},
};
use crate::history::public_value_inclusion::prove_public_value_inclusion;
use dep::std::option::Option;
use dep::protocol_types::traits::{Deserialize, Serialize};

struct StablePublicState<T>{
    context: Context,
    storage_slot: Field,
}

impl<T> StablePublicState<T> {
    pub fn new(
        // Note: Passing the contexts to new(...) just to have an interface compatible with a Map.
        context: Context,
        storage_slot: Field
    ) -> Self {
        assert(storage_slot != 0, "Storage slot 0 not allowed. Storage slots must start from 1.");
        Self {
            context,
            storage_slot,
        }
    }

    // Intended to be only called once. 
    pub fn initialize<T_SERIALIZED_LEN>(self, value: T) where T: Serialize<T_SERIALIZED_LEN> {
        assert(self.context.private.is_none(), "Public state wrties only supported in public functions");
        // TODO: Must throw if the storage slot is not empty -> cannot allow overwriting
        // This is currently impractical, as public functions are never marked `is_contract_deployment`
        // in the `call_context`, only private functions will have this flag set.
        let fields = T::serialize(value);
        storage_write(self.storage_slot, fields);
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

        // TODO: The block_number here can be removed when using the current header in the membership proof.
        let block_number = private_context.get_header().global_variables.block_number;

        // Loop over the fields and prove their inclusion in the public tree
        for i in 0..fields.len() {
            // TODO: Update membership proofs to use current header (Requires #4179)
            // Currently executing unnecessary computation:
            // - a membership proof of the header(block_number) in the history
            // - a membership proof of the value in the public tree of the header 
            prove_public_value_inclusion(
                fields[i],
                self.storage_slot + i,
                block_number as u32,
                (*private_context),
            )
        }
        T::deserialize(fields)
    }

}
