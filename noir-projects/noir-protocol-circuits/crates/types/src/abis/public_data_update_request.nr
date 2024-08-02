use crate::constants::{PUBLIC_DATA_UPDATE_REQUEST_LENGTH, GENERATOR_INDEX__PUBLIC_DATA_UPDATE_REQUEST};
use crate::traits::{Empty, Hash, Serialize, Deserialize};
use crate::public_data_tree_leaf::PublicDataTreeLeaf;
use crate::address::AztecAddress;
use crate::contrakt::storage_update_request::StorageUpdateRequest;
use crate::data::hash::{compute_public_data_tree_value, compute_public_data_tree_index};
use crate::abis::side_effect::{Ordered, Positioned};

struct PublicDataUpdateRequest {
    leaf_slot : Field,
    new_value : Field,
    counter: u32
}

impl PublicDataUpdateRequest {
    pub fn from_contract_storage_update_request(
        contract_address: AztecAddress,
        update_request: StorageUpdateRequest
    ) -> PublicDataUpdateRequest {
        PublicDataUpdateRequest {
            leaf_slot: compute_public_data_tree_index(contract_address, update_request.storage_slot),
            new_value: compute_public_data_tree_value(update_request.new_value),
            counter: update_request.counter
        }
    }
}

impl Positioned for PublicDataUpdateRequest {
    fn position(self) -> Field {
        self.leaf_slot
    }
}

impl Ordered for PublicDataUpdateRequest {
    fn counter(self)-> u32{
        self.counter
    }
}

impl Eq for PublicDataUpdateRequest {
    fn eq(self, update_request: PublicDataUpdateRequest) -> bool {
        (update_request.leaf_slot == self.leaf_slot)
            & (update_request.new_value == self.new_value)
    }
}

impl Empty for PublicDataUpdateRequest {
    fn empty() -> Self {
        Self {
            leaf_slot : 0,
            new_value : 0,
            counter : 0
        }
    }
}

impl Hash for PublicDataUpdateRequest {
    fn hash(self) -> Field {
        crate::hash::poseidon2_hash_with_separator([
            self.leaf_slot,
            self.new_value
        ], GENERATOR_INDEX__PUBLIC_DATA_UPDATE_REQUEST)
    }
}

impl From<PublicDataUpdateRequest> for PublicDataTreeLeaf {
    fn from(update_request: PublicDataUpdateRequest) -> PublicDataTreeLeaf {
        PublicDataTreeLeaf {
            slot: update_request.leaf_slot,
            value: update_request.new_value
        }
    }

}

impl PublicDataUpdateRequest {
    pub fn is_empty(self) -> bool {
        (self.leaf_slot == 0) & (self.new_value == 0)
    }
}

impl Serialize<PUBLIC_DATA_UPDATE_REQUEST_LENGTH> for PublicDataUpdateRequest {
  fn serialize(self) -> [Field; PUBLIC_DATA_UPDATE_REQUEST_LENGTH] {
    [self.leaf_slot, self.new_value, self.counter as Field]
  }
}

impl Deserialize<PUBLIC_DATA_UPDATE_REQUEST_LENGTH> for PublicDataUpdateRequest {
  fn deserialize(fields: [Field; PUBLIC_DATA_UPDATE_REQUEST_LENGTH]) -> PublicDataUpdateRequest {
    PublicDataUpdateRequest {
      leaf_slot: fields[0],
      new_value: fields[1],
        counter: fields[2] as u32
    }
  }
}

#[test]
fn serialization_of_empty() {
    let item = PublicDataUpdateRequest::empty();
    let serialized = item.serialize();
    let deserialized = PublicDataUpdateRequest::deserialize(serialized);
    assert(item.eq(deserialized));
}
