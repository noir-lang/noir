use crate::{
    abis::append_only_tree_snapshot::AppendOnlyTreeSnapshot,
    merkle_tree::{membership::assert_check_membership, root::root_from_sibling_path},
};

pub fn insert_subtree_to_snapshot_tree<let N: u32>(
    snapshot: AppendOnlyTreeSnapshot,
    sibling_path: [Field; N],
    empty_subtree_root: Field,
    subtree_root_to_insert: Field,
    subtree_depth: u8,
) -> AppendOnlyTreeSnapshot {
    // TODO(Lasse): Sanity check len of sibling_path > height of subtree
    // TODO(Lasse): Ensure height of subtree is correct (eg 3 for commitments, 1 for contracts)
    let leaf_index_at_depth = snapshot.next_available_leaf_index >> (subtree_depth as u8);

    // Check that the current root is correct and that there is an empty subtree at the insertion location
    assert_check_membership(
        empty_subtree_root,
        leaf_index_at_depth as Field,
        sibling_path,
        snapshot.root,
    );

    // if index of leaf is x, index of its parent is x/2 or x >> 1. We need to find the parent `subtree_depth` levels up.
    let new_root = root_from_sibling_path(
        subtree_root_to_insert,
        leaf_index_at_depth as Field,
        sibling_path,
    );

    // 2^subtree_depth is the number of leaves added. 2^x = 1 << x
    let new_next_available_leaf_index =
        (snapshot.next_available_leaf_index as u64) + (1 << (subtree_depth as u8));

    AppendOnlyTreeSnapshot {
        root: new_root,
        next_available_leaf_index: new_next_available_leaf_index as u32,
    }
}
