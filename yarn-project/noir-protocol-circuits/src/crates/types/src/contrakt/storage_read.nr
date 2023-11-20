use dep::aztec::constants_gen;

struct StorageRead{
    storage_slot: Field,
    current_value: Field,
}

impl StorageRead {
    pub fn empty() -> Self {
        Self {
            storage_slot: 0,
            current_value: 0,
        }
    }

    pub fn hash(self) -> Field {
        dep::std::hash::pedersen_hash_with_separator([
            self.storage_slot,
            self.current_value,
        ], constants_gen::GENERATOR_INDEX__PUBLIC_DATA_READ)
    }

    pub fn is_empty(self) -> bool {
        self.storage_slot == 0
    }
}
