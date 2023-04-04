use crate::set::Set;

struct Map {
    storage_slot: Field,
}

impl Map {
    fn new(storage_slot: Field) -> Map {
        Map { storage_slot }
    }

    fn at(self, key: Field) -> Set {
        let derived_storage_slot = dep::std::hash::pedersen([
            crate::MappingStorageSlot, 
            self.storage_slot, 
            key
        ])[0];

        Set::new(derived_storage_slot)
    }
}
