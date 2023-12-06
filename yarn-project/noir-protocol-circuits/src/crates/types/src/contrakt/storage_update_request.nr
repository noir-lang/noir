use crate::constants::GENERATOR_INDEX__PUBLIC_DATA_UPDATE_REQUEST;

struct StorageUpdateRequest{
    storage_slot : Field,
    old_value : Field,
    new_value : Field,
}

impl StorageUpdateRequest {
    pub fn empty() -> Self {
        StorageUpdateRequest {
            storage_slot: 0,
            old_value: 0,
            new_value: 0,
        }
    }

    pub fn hash(self) -> Field {
        dep::std::hash::pedersen_hash_with_separator([
            self.storage_slot,
            self.old_value,
            self.new_value,
        ], GENERATOR_INDEX__PUBLIC_DATA_UPDATE_REQUEST)
    }

    pub fn is_empty(self) -> bool {
        self.storage_slot == 0
    }

    pub fn eq(self, request: Self) -> bool {
        (request.storage_slot == self.storage_slot)
            & (request.old_value == self.old_value)
            & (request.new_value == self.new_value)
    }
}
