use crate::{
    constants::{CONTRACT_STORAGE_UPDATE_REQUEST_LENGTH, GENERATOR_INDEX__PUBLIC_DATA_UPDATE_REQUEST},
    hash::poseidon2_hash_with_separator, traits::{Deserialize, Hash, Empty, Serialize}
};

struct StorageUpdateRequest {
    storage_slot : Field,
    new_value : Field,
    counter: u32
}

impl Eq for StorageUpdateRequest {
    fn eq(self, request: Self) -> bool {
        (request.storage_slot == self.storage_slot)
            & (request.new_value == self.new_value)
    }
}

impl Empty for StorageUpdateRequest {
    fn empty() -> Self {
        StorageUpdateRequest {
            storage_slot: 0,
            new_value: 0,
            counter: 0
        }
    }
}

impl Hash for StorageUpdateRequest {
    fn hash(self) -> Field {
        poseidon2_hash_with_separator(self.serialize(), GENERATOR_INDEX__PUBLIC_DATA_UPDATE_REQUEST)
    }
}

impl Serialize<CONTRACT_STORAGE_UPDATE_REQUEST_LENGTH> for StorageUpdateRequest {
    fn serialize(self) -> [Field; CONTRACT_STORAGE_UPDATE_REQUEST_LENGTH] {
        [self.storage_slot, self.new_value, self.counter as Field]
    }
}

impl Deserialize<CONTRACT_STORAGE_UPDATE_REQUEST_LENGTH> for StorageUpdateRequest {
    fn deserialize(serialized: [Field; CONTRACT_STORAGE_UPDATE_REQUEST_LENGTH]) -> Self {
        StorageUpdateRequest {
            storage_slot: serialized[0],
            new_value: serialized[1],
            counter: serialized[2] as u32
        }
    }
}

impl StorageUpdateRequest {
    pub fn is_empty(self) -> bool {
        self.storage_slot == 0
    }
}

#[test]
fn serialization_of_empty() {
    let item = StorageUpdateRequest::empty();
    let serialized = item.serialize();
    let deserialized = StorageUpdateRequest::deserialize(serialized);
    assert(item.eq(deserialized));
}
