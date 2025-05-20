use crate::types::{
    merkle_tree::{
        indexed_tree::check_valid_low_leaf::assert_check_valid_low_leaf,
        leaf_preimage::IndexedTreeLeafPreimage, root::root_from_sibling_path,
    },
    traits::Empty,
};

pub(crate) struct MembershipWitness<let N: u32> {
    pub(crate) leaf_index: Field,
    pub(crate) sibling_path: [Field; N],
}

impl<let N: u32> Empty for MembershipWitness<N> {
    fn empty() -> Self {
        MembershipWitness { leaf_index: 0, sibling_path: [0; N] }
    }
}

pub(crate) fn check_membership<let N: u32>(
    leaf: Field,
    index: Field,
    sibling_path: [Field; N],
    root: Field,
) -> bool {
    let calculated_root = root_from_sibling_path(leaf, index, sibling_path);
    calculated_root == root
}

pub(crate) fn assert_check_membership<let TREE_HEIGHT: u32>(
    leaf: Field,
    index: Field,
    sibling_path: [Field; TREE_HEIGHT],
    root: Field,
) {
    assert(check_membership(leaf, index, sibling_path, root), "membership check failed");
}

pub(crate) fn assert_check_non_membership<let TREE_HEIGHT: u32, LEAF_PREIMAGE, VALUE>(
    key: Field,
    low_leaf_preimage: LEAF_PREIMAGE,
    low_leaf_membership_witness: MembershipWitness<TREE_HEIGHT>,
    tree_root: Field,
)
where
    LEAF_PREIMAGE: IndexedTreeLeafPreimage<VALUE>,
{
    assert_check_valid_low_leaf(key, low_leaf_preimage);

    let low_leaf_exists = check_membership(
        low_leaf_preimage.as_leaf(),
        low_leaf_membership_witness.leaf_index,
        low_leaf_membership_witness.sibling_path,
        tree_root,
    );
    assert(low_leaf_exists, "Low leaf does not exist");
}

// Prove either membership or non-membership depending on the value of `exists`.
// If `exists` == false, `key` is not in the tree, `leaf_preimage` and `membership_witness` are for the low leaf.
pub(crate) fn conditionally_assert_check_membership<let TREE_HEIGHT: u32, LEAF_PREIMAGE, VALUE>(
    key: Field,
    exists: bool,
    leaf_preimage: LEAF_PREIMAGE,
    membership_witness: MembershipWitness<TREE_HEIGHT>,
    tree_root: Field,
)
where
    LEAF_PREIMAGE: IndexedTreeLeafPreimage<VALUE>,
{
    if exists {
        assert(key == leaf_preimage.get_key(), "Key does not match the key of the leaf preimage");
    } else {
        assert_check_valid_low_leaf(key, leaf_preimage);
    }

    assert_check_membership(
        leaf_preimage.as_leaf(),
        membership_witness.leaf_index,
        membership_witness.sibling_path,
        tree_root,
    );
}
