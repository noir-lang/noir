use crate::context::{PrivateContext, PublicContext, Context};
use dep::std::option::Option;
use dep::protocol_types::hash::pedersen_hash;

// docs:start:map
struct Map<V> {
    context: Context,
    storage_slot: Field,
    state_var_constructor: fn(Context, Field) -> V,
}
// docs:end:map

impl<V> Map<V> {
    // docs:start:new
    pub fn new(
        context: Context,
        storage_slot: Field,
        state_var_constructor: fn(Context, Field) -> V,
    ) -> Map<V> {
        assert(storage_slot != 0, "Storage slot 0 not allowed. Storage slots must start from 1.");
        Map {
            context,
            storage_slot,
            state_var_constructor,
        }
    }
    // docs:end:new

    // docs:start:at
    pub fn at(self, key: Field) -> V {
        // TODO(#1204): use a generator index for the storage slot
        let derived_storage_slot = pedersen_hash([self.storage_slot, key],0);

        let state_var_constructor = self.state_var_constructor;
        state_var_constructor(self.context, derived_storage_slot)
    }
    // docs:end:at
}
