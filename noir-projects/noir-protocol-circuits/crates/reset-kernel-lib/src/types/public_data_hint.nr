use crate::reset::{mutable_data_read_request::LeafDataHint};
use dep::types::{
    public_data_tree_leaf_preimage::PublicDataTreeLeafPreimage, merkle_tree::MembershipWitness,
    constants::PUBLIC_DATA_TREE_HEIGHT
};

struct PublicDataHint {
    leaf_slot: Field,
    value: Field,
    override_counter: u32,
    membership_witness: MembershipWitness<PUBLIC_DATA_TREE_HEIGHT>, // Should be MembershipWitness<PUBLIC_DATA_TREE_HEIGHT> when we can handle generics when converting to ts types.
    leaf_preimage: PublicDataTreeLeafPreimage,
}

impl LeafDataHint for PublicDataHint {
    fn leaf_slot(self) -> Field {
        self.leaf_slot
    }

    fn value(self) -> Field {
        self.value
    }
    
    fn override_counter(self) -> u32 {
        self.override_counter
    }
}
