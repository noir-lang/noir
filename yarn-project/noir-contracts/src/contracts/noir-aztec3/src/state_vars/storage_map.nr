use crate::state_vars::storage_field::StorageField;

// TODO: Refactor using generics and merge with `map.nr`
struct StorageMap {
    storage_slot: Field,
}

impl StorageMap {
    fn new(storage_slot: Field) -> StorageMap {
        StorageMap { storage_slot }
    }

    fn at(self, key: Field) -> StorageField {
        let derived_storage_slot = dep::std::hash::pedersen([
            crate::MappingStorageSlot, 
            self.storage_slot, 
            key
        ])[0];

        StorageField::new(derived_storage_slot)
    }
}
