use crate::constants::GENERATOR_INDEX__PUBLIC_DATA_READ;
use dep::std::cmp::Eq;
use crate::traits::{Empty, Hash};

struct PublicDataRead {
    leaf_slot : Field,
    value : Field,
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
        dep::std::hash::pedersen_hash_with_separator([
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
