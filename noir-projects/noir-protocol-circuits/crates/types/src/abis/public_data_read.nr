use crate::constants::{GENERATOR_INDEX__PUBLIC_DATA_READ, PUBLIC_DATA_READ_LENGTH};
use crate::traits::{Empty, Hash, Serialize, Deserialize};
use crate::contrakt::storage_read::StorageRead;
use crate::data::hash::{compute_public_data_tree_value, compute_public_data_tree_index};
use crate::address::AztecAddress;

struct PublicDataRead {
    leaf_slot : Field,
    value : Field,
}

impl PublicDataRead {
    pub fn from_contract_storage_read(
        contract_address: AztecAddress,
        read_request: StorageRead
    ) -> PublicDataRead {
        PublicDataRead {
            leaf_slot: compute_public_data_tree_index(contract_address, read_request.storage_slot),
            value: compute_public_data_tree_value(read_request.current_value)
        }
    }
}

impl Eq for PublicDataRead {
    fn eq(self, public_data_read: PublicDataRead) -> bool {
        (public_data_read.leaf_slot == self.leaf_slot) & (public_data_read.value == self.value)
    }
}

impl Empty for PublicDataRead {
    fn empty() -> Self {
        Self {
            leaf_slot : 0,
            value : 0,
        }
    }
}

impl Hash for PublicDataRead {
    fn hash(self) -> Field {
        crate::hash::poseidon2_hash_with_separator([
            self.leaf_slot,
            self.value,
        ], GENERATOR_INDEX__PUBLIC_DATA_READ)
    }
}

impl PublicDataRead {
    pub fn is_empty(self) -> bool {
        (self.leaf_slot == 0) & (self.value == 0)
    }
}

impl Serialize<PUBLIC_DATA_READ_LENGTH> for PublicDataRead {
  fn serialize(self) -> [Field; PUBLIC_DATA_READ_LENGTH] {
    [self.leaf_slot, self.value]
  }
}

impl Deserialize<PUBLIC_DATA_READ_LENGTH> for PublicDataRead {
  fn deserialize(fields: [Field; PUBLIC_DATA_READ_LENGTH]) -> Self {
    PublicDataRead {
      leaf_slot: fields[0],
      value: fields[1],
    }
  }
}

#[test]
fn serialization_of_empty() {
    let item = PublicDataRead::empty();
    let serialized = item.serialize();
    let deserialized = PublicDataRead::deserialize(serialized);
    assert(item.eq(deserialized));
}
