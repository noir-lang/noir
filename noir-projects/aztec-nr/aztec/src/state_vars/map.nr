use dep::protocol_types::{hash::pedersen_hash, storage::map::derive_storage_slot_in_map, traits::ToField};
use crate::state_vars::storage::Storage;

// docs:start:map
struct Map<K, V, Context> {
    context: Context,
    storage_slot: Field,
    state_var_constructor: fn(Context, Field) -> V,
}
// docs:end:map

impl<K, T, Context> Storage<T> for Map<K, T, Context> {}

impl<K, V, Context> Map<K, V, Context> {
    // docs:start:new
    pub fn new(
        context: Context,
        storage_slot: Field,
        state_var_constructor: fn(Context, Field) -> V
    ) -> Self {
        assert(storage_slot != 0, "Storage slot 0 not allowed. Storage slots must start from 1.");
        Map { context, storage_slot, state_var_constructor }
    }
    // docs:end:new

    // docs:start:at
    pub fn at(self, key: K) -> V where K: ToField {
        // TODO(#1204): use a generator index for the storage slot
        let derived_storage_slot = derive_storage_slot_in_map(self.storage_slot, key);

        let state_var_constructor = self.state_var_constructor;
        state_var_constructor(self.context, derived_storage_slot)
    }
    // docs:end:at
}
