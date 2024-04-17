use crate::{merkle_tree::leaf_preimage::IndexedTreeLeafPreimage, traits::{Empty, Hash}};

struct PublicDataTreeLeafPreimage {
    slot : Field,
    value: Field,
    next_slot :Field,
    next_index : u64,
}

impl Empty for PublicDataTreeLeafPreimage {
    fn empty() -> Self {
        Self {
            slot: 0,
            value: 0,
            next_slot: 0,
            next_index: 0,
        }
    }
}

impl Hash for PublicDataTreeLeafPreimage {
    fn hash(self) -> Field {
        if self.is_empty() {
            0
        } else {
            dep::std::hash::pedersen_hash([self.slot, self.value, (self.next_index as Field), self.next_slot])
        }
    }
}

impl IndexedTreeLeafPreimage for PublicDataTreeLeafPreimage {
    fn get_key(self) -> Field {
        self.slot
    }

    fn get_next_key(self) -> Field {
        self.next_slot
    }

    fn as_leaf(self) -> Field {
        self.hash()
    }
}

impl PublicDataTreeLeafPreimage {
    pub fn is_empty(self) -> bool {
        (self.slot == 0) & (self.value == 0) & (self.next_slot == 0) & (self.next_index == 0)
    }
}
