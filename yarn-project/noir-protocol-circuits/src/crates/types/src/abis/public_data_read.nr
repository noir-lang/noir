use crate::constants::GENERATOR_INDEX__PUBLIC_DATA_READ;

struct PublicDataRead {
    leaf_slot : Field,
    value : Field,
}

impl PublicDataRead {
    fn hash(self) -> Field {
        dep::std::hash::pedersen_hash_with_separator([
            self.leaf_slot,
            self.value,
        ], GENERATOR_INDEX__PUBLIC_DATA_READ)
    }

    pub fn empty() -> Self {
        Self {
            leaf_slot : 0,
            value : 0,
        }
    }

    pub fn is_empty(self) -> bool {
        (self.leaf_slot == 0) & (self.value == 0)
    }

    pub fn eq(self, public_data_read: PublicDataRead) -> bool {
        (public_data_read.leaf_slot == self.leaf_slot) & (public_data_read.value == self.value)
    }
}
