use crate::{
    constants::{
        CONTRACT_STORAGE_UPDATE_REQUEST_LENGTH,
        GENERATOR_INDEX__PUBLIC_DATA_UPDATE_REQUEST,
    },
    hash::pedersen_hash,
    traits::{
        Deserialize,
        Hash,
        Empty,
        Serialize,
    },
};
use dep::std::cmp::Eq;

struct StorageUpdateRequest{
    storage_slot : Field,
    old_value : Field,
    new_value : Field,
}

impl Eq for StorageUpdateRequest {
    fn eq(self, request: Self) -> bool {
        // TODO(https://github.com/AztecProtocol/aztec-packages/issues/3595)
        (request.storage_slot == self.storage_slot)
            & (request.old_value == self.old_value)
            & (request.new_value == self.new_value)
    }
}

impl Empty for StorageUpdateRequest {
    fn empty() -> Self {
        StorageUpdateRequest {
            storage_slot: 0,
            old_value: 0,
            new_value: 0,
        }
    }
}

impl Hash for StorageUpdateRequest {
    fn hash(self) -> Field {
        pedersen_hash(self.serialize(), GENERATOR_INDEX__PUBLIC_DATA_UPDATE_REQUEST)
    }
}

impl Serialize<CONTRACT_STORAGE_UPDATE_REQUEST_LENGTH> for StorageUpdateRequest {
    fn serialize(self) -> [Field; CONTRACT_STORAGE_UPDATE_REQUEST_LENGTH] {
        [self.storage_slot, self.old_value, self.new_value]
    }
}

impl Deserialize<CONTRACT_STORAGE_UPDATE_REQUEST_LENGTH> for StorageUpdateRequest {
    fn deserialize(serialized: [Field; CONTRACT_STORAGE_UPDATE_REQUEST_LENGTH]) -> Self {
        StorageUpdateRequest {
            storage_slot: serialized[0],
            old_value: serialized[1],
            new_value: serialized[2],
        }
    }
}

impl StorageUpdateRequest {
    pub fn is_empty(self) -> bool {
        self.storage_slot == 0
    }
}
