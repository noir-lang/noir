use crate::{
    constants::PUBLIC_DATA_TREE_HEIGHT,
    data::public_data_tree_leaf_preimage::PublicDataTreeLeafPreimage,
    merkle_tree::{conditionally_assert_check_membership, MembershipWitness},
    traits::Empty,
};

pub struct PublicDataHint {
    pub leaf_slot: Field,
    pub value: Field,
    pub membership_witness: MembershipWitness<PUBLIC_DATA_TREE_HEIGHT>,
    pub leaf_preimage: PublicDataTreeLeafPreimage,
}

impl PublicDataHint {
    pub fn validate(self, public_data_tree_root: Field) {
        // We only need to check leaf_slot to decide if a (non-)membership check is required.
        // It will fail if a PublicDataHint with 0 leaf_slot is used to verify a non-empty public read or write.
        if self.leaf_slot != 0 {
            let exists_in_tree = self.leaf_slot == self.leaf_preimage.slot;
            if exists_in_tree {
                assert(
                    self.value == self.leaf_preimage.value,
                    "Hinted public data value does not match the value in leaf preimage",
                );
            } else {
                assert(self.value == 0, "Value must be 0 for non-existent public data");
            }

            conditionally_assert_check_membership(
                self.leaf_slot,
                exists_in_tree,
                self.leaf_preimage,
                self.membership_witness,
                public_data_tree_root,
            );
        }
    }
}

impl Empty for PublicDataHint {
    fn empty() -> Self {
        PublicDataHint {
            leaf_slot: 0,
            value: 0,
            membership_witness: MembershipWitness::empty(),
            leaf_preimage: PublicDataTreeLeafPreimage::empty(),
        }
    }
}
