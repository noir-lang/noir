use crate::constants::GENERATOR_INDEX__PUBLIC_DATA_UPDATE_REQUEST;

struct PublicDataUpdateRequest {
    leaf_index : Field,
    old_value : Field,
    new_value : Field
}

impl PublicDataUpdateRequest {
    pub fn empty() -> Self {
        Self {
            leaf_index : 0,
            old_value : 0,
            new_value : 0
        }
    }

    pub fn eq(self, update_request: PublicDataUpdateRequest) -> bool {
        (update_request.leaf_index == self.leaf_index) & (update_request.old_value == self.old_value) 
            & (update_request.new_value == self.new_value)
    }

    pub fn hash(self) -> Field {
        dep::std::hash::pedersen_hash_with_separator([
            self.leaf_index,
            self.old_value,
            self.new_value
        ], GENERATOR_INDEX__PUBLIC_DATA_UPDATE_REQUEST)
    }

    pub fn is_empty(self) -> bool {
        (self.leaf_index == 0) & (self.old_value == 0) & (self.new_value == 0)
    }
}
