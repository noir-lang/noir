use crate::constants::{PUBLIC_DATA_UPDATE_REQUEST_LENGTH, GENERATOR_INDEX__PUBLIC_DATA_UPDATE_REQUEST};
use dep::std::cmp::Eq;
use crate::traits::{Empty, Hash, Serialize, Deserialize};

struct PublicDataUpdateRequest {
    leaf_slot : Field,
    new_value : Field
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
            new_value : 0
        }
    }
}

impl Hash for PublicDataUpdateRequest {
    fn hash(self) -> Field {
        dep::std::hash::pedersen_hash_with_separator([
            self.leaf_slot,
            self.new_value
        ], GENERATOR_INDEX__PUBLIC_DATA_UPDATE_REQUEST)
    }
}

impl PublicDataUpdateRequest {
    pub fn is_empty(self) -> bool {
        (self.leaf_slot == 0) & (self.new_value == 0)
    }
}

impl Serialize<PUBLIC_DATA_UPDATE_REQUEST_LENGTH> for PublicDataUpdateRequest {
  fn serialize(self) -> [Field; PUBLIC_DATA_UPDATE_REQUEST_LENGTH] {
    [self.leaf_slot, self.new_value]
  }
}

impl Deserialize<PUBLIC_DATA_UPDATE_REQUEST_LENGTH> for PublicDataUpdateRequest {
  fn deserialize(fields: [Field; PUBLIC_DATA_UPDATE_REQUEST_LENGTH]) -> PublicDataUpdateRequest {
    PublicDataUpdateRequest {
      leaf_slot: fields[0],
      new_value: fields[1],
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
