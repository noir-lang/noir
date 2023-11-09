use crate::{
    root::{
        root_rollup_inputs::RootRollupInputs,
    },
};
use dep::aztec::constants_gen::{
    L1_TO_L2_MSG_TREE_HEIGHT,
    L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH,
    L1_TO_L2_MSG_SUBTREE_HEIGHT,
    HISTORIC_BLOCKS_TREE_HEIGHT,
};
use crate::abis::append_only_tree_snapshot::AppendOnlyTreeSnapshot;
use crate::fixtures::previous_rollup_data::default_previous_rollup_data;

pub fn compute_zero_hashes<N>(mut hashes: [Field; N]) -> [Field; N] {
    hashes[0] = dep::std::hash::pedersen_hash([0, 0]);

    for i in 1..N {
        hashes[i] = dep::std::hash::pedersen_hash([hashes[i-1], hashes[i-1]]);
    }

    hashes
}

pub fn compute_l1_l2_empty_snapshot() -> (AppendOnlyTreeSnapshot, [Field; L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH]) {
    let zero_hashes = compute_zero_hashes([0; L1_TO_L2_MSG_TREE_HEIGHT]);
    let mut new_l1_to_l2_messages_tree_root_sibling_path = [0; L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH];

    for i in 0..L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH {
        let index = L1_TO_L2_MSG_SUBTREE_HEIGHT + i - 1;
        new_l1_to_l2_messages_tree_root_sibling_path[i] = zero_hashes[index];
    }

    (AppendOnlyTreeSnapshot{ root: zero_hashes[zero_hashes.len() - 1], next_available_leaf_index: 0 }, new_l1_to_l2_messages_tree_root_sibling_path)
}

pub fn compute_historic_blocks_tree_snapshot() -> (AppendOnlyTreeSnapshot, [Field; HISTORIC_BLOCKS_TREE_HEIGHT]) {
    let zero_hashes = compute_zero_hashes([0; HISTORIC_BLOCKS_TREE_HEIGHT]);
    let mut sibling_path = [0; HISTORIC_BLOCKS_TREE_HEIGHT];
    for i in 1..HISTORIC_BLOCKS_TREE_HEIGHT {
        sibling_path[i] = zero_hashes[i-1];
    }
    (AppendOnlyTreeSnapshot { root: zero_hashes[zero_hashes.len() - 1], next_available_leaf_index: 0 }, sibling_path)
}


pub fn default_root_rollup_inputs() -> RootRollupInputs {
    let mut inputs: RootRollupInputs = dep::std::unsafe::zeroed();
    let (l1_l2_empty_snapshot, l1_l2_empty_sibling_path) = compute_l1_l2_empty_snapshot();

    inputs.new_l1_to_l2_messages_tree_root_sibling_path = l1_l2_empty_sibling_path;
    inputs.start_l1_to_l2_messages_tree_snapshot = l1_l2_empty_snapshot;

    let (historic_blocks_snapshot, historic_blocks_sibling_path) = compute_historic_blocks_tree_snapshot();

    inputs.start_historic_blocks_tree_snapshot = historic_blocks_snapshot;
    inputs.new_historic_blocks_tree_sibling_path = historic_blocks_sibling_path;

    inputs.previous_rollup_data = default_previous_rollup_data();

    inputs
}