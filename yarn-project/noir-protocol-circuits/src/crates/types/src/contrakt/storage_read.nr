use crate::{
    constants::{
        CONTRACT_STORAGE_READ_LENGTH,
        GENERATOR_INDEX__PUBLIC_DATA_READ,
    },
    hash::pedersen_hash,
};

struct StorageRead {
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

    pub fn serialize(self) -> [Field; CONTRACT_STORAGE_READ_LENGTH] {
        [self.storage_slot, self.current_value]
    }

    pub fn hash(self) -> Field {
        pedersen_hash(self.serialize(), GENERATOR_INDEX__PUBLIC_DATA_READ)
    }

    pub fn is_empty(self) -> bool {
        self.storage_slot == 0
    }
}
