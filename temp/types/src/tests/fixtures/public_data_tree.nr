use crate::{
    constants::PUBLIC_DATA_TREE_HEIGHT,
    data::public_data_tree_leaf_preimage::PublicDataTreeLeafPreimage,
    tests::merkle_tree_utils::NonEmptyMerkleTree, traits::Hash,
};

pub fn empty_public_data_tree<let LEAVES_COUNT: u32, let SUBTREE_HEIGHT: u32>(
    prefill_count: u32,
    ) -> NonEmptyMerkleTree<LEAVES_COUNT, PUBLIC_DATA_TREE_HEIGHT, PUBLIC_DATA_TREE_HEIGHT - SUBTREE_HEIGHT, SUBTREE_HEIGHT> {
    let mut leaves = [0; LEAVES_COUNT];
    for i in 0..(prefill_count - 1) {
        let leaf = PublicDataTreeLeafPreimage {
            slot: i as Field,
            value: 0,
            next_slot: (i + 1) as Field,
            next_index: i + 1,
        };
        leaves[i] = leaf.hash();
    }

    let last_leaf = PublicDataTreeLeafPreimage {
        slot: (prefill_count - 1) as Field,
        value: 0,
        next_slot: 0,
        next_index: 0,
    };
    leaves[prefill_count - 1] = last_leaf.hash();

    NonEmptyMerkleTree::new(
        leaves,
        [0; PUBLIC_DATA_TREE_HEIGHT],
        [0; PUBLIC_DATA_TREE_HEIGHT - SUBTREE_HEIGHT],
        [0; SUBTREE_HEIGHT],
    )
}
