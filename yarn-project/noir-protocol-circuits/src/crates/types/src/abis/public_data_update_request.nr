use dep::aztec::constants_gen;

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

    pub fn is_empty(self) -> bool {
        (self.leaf_index == 0) & (self.old_value == 0) & (self.new_value == 0)
    }

    fn hash(self) -> Field {
        dep::std::hash::pedersen_hash_with_separator([
            self.leaf_index,
            self.old_value,
            self.new_value
        ], constants_gen::GENERATOR_INDEX__PUBLIC_DATA_UPDATE_REQUEST)
    }
}
