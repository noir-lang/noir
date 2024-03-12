use crate::{
    abis::{append_only_tree_snapshot::AppendOnlyTreeSnapshot},
    merkle_tree::{membership::assert_check_membership, root::root_from_sibling_path}
};

pub fn insert_subtree_to_snapshot_tree<N>(
    snapshot: AppendOnlyTreeSnapshot,
    siblingPath: [Field; N],
    emptySubtreeRoot: Field,
    subtreeRootToInsert: Field,
    subtreeDepth: u8
) -> AppendOnlyTreeSnapshot {
    // TODO(Lasse): Sanity check len of siblingPath > height of subtree
    // TODO(Lasse): Ensure height of subtree is correct (eg 3 for commitments, 1 for contracts)
    let leafIndexAtDepth = snapshot.next_available_leaf_index >> (subtreeDepth as u32);

    // Check that the current root is correct and that there is an empty subtree at the insertion location
    assert_check_membership(
        emptySubtreeRoot,
        leafIndexAtDepth as Field,
        siblingPath,
        snapshot.root
    );

    // if index of leaf is x, index of its parent is x/2 or x >> 1. We need to find the parent `subtreeDepth` levels up.
    let new_root = root_from_sibling_path(subtreeRootToInsert, leafIndexAtDepth as Field, siblingPath);

    // 2^subtreeDepth is the number of leaves added. 2^x = 1 << x
    let new_next_available_leaf_index = (snapshot.next_available_leaf_index as u64) + (1 << (subtreeDepth as u64));

    AppendOnlyTreeSnapshot { root: new_root, next_available_leaf_index: new_next_available_leaf_index as u32 }
}
