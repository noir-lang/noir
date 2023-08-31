struct Map<V> {
    storage_slot: Field,
    state_var_constructor: fn (Field) -> V,
}

impl<V> Map<V> {
    fn new(storage_slot: Field, state_var_constructor: fn (Field) -> V) -> Map<V> {
        assert(storage_slot != 0); // Storage slot 0 not allowed. Storage slots must start from 1.
        Map { storage_slot, state_var_constructor }
    }

    fn at(self, key: Field) -> V {
        // TODO(#1204): use a generator index for the storage slot
        let derived_storage_slot = dep::std::hash::pedersen([
            self.storage_slot, 
            key,
        ])[0];

        let state_var_constructor = self.state_var_constructor; 
        state_var_constructor(derived_storage_slot)
    }
}
