use crate::{
    data::public_data_tree_leaf::PublicDataTreeLeaf,
    merkle_tree::leaf_preimage::{IndexedTreeLeafPreimage, LeafPreimage},
    traits::{Empty, Hash},
};

pub struct PublicDataTreeLeafPreimage {
    pub slot: Field,
    pub value: Field,
    pub next_slot: Field,
    pub next_index: u32,
}

impl Empty for PublicDataTreeLeafPreimage {
    fn empty() -> Self {
        Self { slot: 0, value: 0, next_slot: 0, next_index: 0 }
    }
}

impl Eq for PublicDataTreeLeafPreimage {
    fn eq(self, other: Self) -> bool {
        (self.slot == other.slot)
            & (self.value == other.value)
            & (self.next_slot == other.next_slot)
            & (self.next_index == other.next_index)
    }
}

impl Hash for PublicDataTreeLeafPreimage {
    fn hash(self) -> Field {
        if self.is_empty() {
            0
        } else {
            crate::hash::poseidon2_hash([
                self.slot,
                self.value,
                (self.next_index as Field),
                self.next_slot,
            ])
        }
    }
}

impl LeafPreimage for PublicDataTreeLeafPreimage {
    fn get_key(self) -> Field {
        self.slot
    }

    fn as_leaf(self) -> Field {
        self.hash()
    }
}

impl IndexedTreeLeafPreimage<PublicDataTreeLeaf> for PublicDataTreeLeafPreimage {
    fn get_next_key(self) -> Field {
        self.next_slot
    }

    fn points_to_infinity(self) -> bool {
        (self.next_slot == 0) & (self.next_index == 0)
    }

    fn update_pointers(self, next_slot: Field, next_index: u32) -> Self {
        Self { slot: self.slot, value: self.value, next_slot, next_index }
    }

    fn update_value(self, write: PublicDataTreeLeaf) -> Self {
        Self {
            slot: self.slot,
            value: write.value,
            next_slot: self.next_slot,
            next_index: self.next_index,
        }
    }

    fn build_insertion_leaf(write: PublicDataTreeLeaf, low_leaf: Self) -> Self {
        Self {
            slot: write.slot,
            value: write.value,
            next_slot: low_leaf.next_slot,
            next_index: low_leaf.next_index,
        }
    }
}

impl PublicDataTreeLeafPreimage {
    pub fn is_empty(self) -> bool {
        (self.slot == 0) & (self.value == 0) & (self.next_slot == 0) & (self.next_index == 0)
    }
}
