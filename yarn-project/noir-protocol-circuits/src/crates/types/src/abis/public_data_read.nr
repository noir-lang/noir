use dep::aztec::constants_gen;

struct PublicDataRead {
    leaf_index : Field,
    value : Field,
}

impl PublicDataRead {
    fn hash(self) -> Field {
        dep::std::hash::pedersen_hash_with_separator([
            self.leaf_index,
            self.value,
        ], constants_gen::GENERATOR_INDEX__PUBLIC_DATA_READ)
    }

    pub fn empty() -> Self {
        Self {
            leaf_index : 0,
            value : 0,
        }
    }

    pub fn is_empty(self) -> bool {
        (self.leaf_index == 0) & (self.value == 0)
    }
}
