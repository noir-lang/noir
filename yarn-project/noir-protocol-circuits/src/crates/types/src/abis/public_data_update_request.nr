use crate::constants::GENERATOR_INDEX__PUBLIC_DATA_UPDATE_REQUEST;
use dep::std::ops::Eq;
use crate::traits::{Empty, Hash};

struct PublicDataUpdateRequest {
    leaf_slot : Field,
    old_value : Field,
    new_value : Field
}

impl Eq for PublicDataUpdateRequest {
    fn eq(self, update_request: PublicDataUpdateRequest) -> bool {
        (update_request.leaf_slot == self.leaf_slot) & (update_request.old_value == self.old_value) 
            & (update_request.new_value == self.new_value)
    }
}

impl Empty for PublicDataUpdateRequest {
    fn empty() -> Self {
        Self {
            leaf_slot : 0,
            old_value : 0,
            new_value : 0
        }
    }
}

impl Hash for PublicDataUpdateRequest {
    fn hash(self) -> Field {
        dep::std::hash::pedersen_hash_with_separator([
            self.leaf_slot,
            self.old_value,
            self.new_value
        ], GENERATOR_INDEX__PUBLIC_DATA_UPDATE_REQUEST)
    }
}

impl PublicDataUpdateRequest {
    pub fn is_empty(self) -> bool {
        (self.leaf_slot == 0) & (self.old_value == 0) & (self.new_value == 0)
    }
}
