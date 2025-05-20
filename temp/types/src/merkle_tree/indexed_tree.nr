pub mod check_valid_low_leaf;

use crate::{
    abis::append_only_tree_snapshot::AppendOnlyTreeSnapshot,
    merkle_tree::{
        leaf_preimage::{IndexedTreeLeafPreimage, IndexedTreeLeafValue},
        membership::{assert_check_membership, MembershipWitness},
        root::{calculate_empty_tree_root, calculate_subtree_root, root_from_sibling_path},
    },
    traits::{Empty, is_empty},
    utils::arrays::check_permutation,
};

pub fn batch_insert<Value, Leaf, let SubtreeWidth: u32, let SiblingPathLength: u32, let SubtreeHeight: u32, let TreeHeight: u32>(
    start_snapshot: AppendOnlyTreeSnapshot,
    values_to_insert: [Value; SubtreeWidth],
    sorted_values: [Value; SubtreeWidth],
    sorted_values_indexes: [u32; SubtreeWidth],
    new_subtree_sibling_path: [Field; SiblingPathLength],
    low_leaf_preimages: [Leaf; SubtreeWidth],
    low_leaf_membership_witnesses: [MembershipWitness<TreeHeight>; SubtreeWidth],
) -> AppendOnlyTreeSnapshot
where
    Value: IndexedTreeLeafValue,
    Leaf: IndexedTreeLeafPreimage<Value>,
{
    // A permutation to the values is provided to make the insertion use only one insertion strategy
    // However, for the actual insertion in the tree the original order is respected, the sorting is only used for validation of the links
    // and low leaf updates.
    check_permutation(values_to_insert, sorted_values, sorted_values_indexes);

    // Now, update the existing leaves with the new leaves
    let mut current_tree_root = start_snapshot.root;
    let mut insertion_subtree = [Empty::empty(); SubtreeWidth];
    let start_insertion_index = start_snapshot.next_available_leaf_index;

    for i in 0..sorted_values.len() {
        let value = sorted_values[i];
        if !is_empty(value) {
            let low_leaf_preimage = low_leaf_preimages[i];
            let witness = low_leaf_membership_witnesses[i];

            // validate the low leaf
            assert(!is_empty(low_leaf_preimage), "Empty low leaf");
            let value_key = value.get_key();
            let low_leaf_key = low_leaf_preimage.get_key();
            let low_leaf_next_key = low_leaf_preimage.get_next_key();
            let is_update = value_key == low_leaf_key;

            let is_less_than_slot = low_leaf_key.lt(value_key);
            let is_next_greater_than = value_key.lt(low_leaf_next_key);
            let is_in_range =
                is_less_than_slot & (is_next_greater_than | low_leaf_preimage.points_to_infinity());

            assert(is_update | is_in_range, "Invalid low leaf");

            // perform membership check for the low leaf against the original root
            assert_check_membership(
                low_leaf_preimage.as_leaf(),
                witness.leaf_index,
                witness.sibling_path,
                current_tree_root,
            );

            let value_index = sorted_values_indexes[i];

            // Calculate the new value of the low_leaf
            let updated_low_leaf = if is_update {
                low_leaf_preimage.update_value(value)
            } else {
                low_leaf_preimage.update_pointers(
                    value_key,
                    start_insertion_index as u32 + value_index,
                )
            };

            current_tree_root = root_from_sibling_path(
                updated_low_leaf.as_leaf(),
                witness.leaf_index,
                witness.sibling_path,
            );

            insertion_subtree[value_index] = if is_update {
                Empty::empty()
            } else {
                Leaf::build_insertion_leaf(value, low_leaf_preimage)
            };
        }
    }

    let empty_subtree_root = calculate_empty_tree_root(SubtreeHeight);
    let leaf_index_subtree_depth = start_insertion_index >> (SubtreeHeight as u8);

    assert_check_membership(
        empty_subtree_root,
        leaf_index_subtree_depth as Field,
        new_subtree_sibling_path,
        current_tree_root,
    );

    // Create new subtree to insert into the whole indexed tree
    let subtree_root = calculate_subtree_root(insertion_subtree.map(|leaf: Leaf| leaf.as_leaf()));

    // Calculate the new root
    // We are inserting a subtree rather than a full tree here
    let subtree_index = start_insertion_index >> (SubtreeHeight as u8);
    let new_root = root_from_sibling_path(
        subtree_root,
        subtree_index as Field,
        new_subtree_sibling_path,
    );

    AppendOnlyTreeSnapshot {
        root: new_root,
        next_available_leaf_index: start_insertion_index + (values_to_insert.len() as u32),
    }
}

pub fn insert<Value, Leaf, let TreeHeight: u32>(
    mut snapshot: AppendOnlyTreeSnapshot,
    value: Value,
    low_leaf_preimage: Leaf,
    low_leaf_membership_witness: MembershipWitness<TreeHeight>,
    insertion_sibling_path: [Field; TreeHeight],
) -> AppendOnlyTreeSnapshot
where
    Value: IndexedTreeLeafValue,
    Leaf: IndexedTreeLeafPreimage<Value>,
{
    // validate the low leaf
    assert(!is_empty(low_leaf_preimage), "Empty low leaf");
    let value_key = value.get_key();
    let low_leaf_key = low_leaf_preimage.get_key();
    let low_leaf_next_key = low_leaf_preimage.get_next_key();
    let is_update = value_key == low_leaf_key;

    let is_less_than_slot = low_leaf_key.lt(value_key);
    let is_next_greater_than = value_key.lt(low_leaf_next_key);
    let is_in_range =
        is_less_than_slot & (is_next_greater_than | low_leaf_preimage.points_to_infinity());

    assert(is_update | is_in_range, "Invalid low leaf");

    // perform membership check for the low leaf against the original root
    assert_check_membership(
        low_leaf_preimage.as_leaf(),
        low_leaf_membership_witness.leaf_index,
        low_leaf_membership_witness.sibling_path,
        snapshot.root,
    );

    // Calculate the new value of the low_leaf
    let updated_low_leaf = if is_update {
        low_leaf_preimage.update_value(value)
    } else {
        low_leaf_preimage.update_pointers(value_key, snapshot.next_available_leaf_index)
    };

    // Update low leaf
    snapshot.root = root_from_sibling_path(
        updated_low_leaf.as_leaf(),
        low_leaf_membership_witness.leaf_index,
        low_leaf_membership_witness.sibling_path,
    );

    if is_update {
        // If it's an update, we don't need to insert the new leaf and advance the tree
        snapshot
    } else {
        let insertion_leaf = Leaf::build_insertion_leaf(value, low_leaf_preimage);
        assert_check_membership(
            0,
            snapshot.next_available_leaf_index as Field,
            insertion_sibling_path,
            snapshot.root,
        );

        // Calculate the new root
        snapshot.root = root_from_sibling_path(
            insertion_leaf.as_leaf(),
            snapshot.next_available_leaf_index as Field,
            insertion_sibling_path,
        );

        snapshot.next_available_leaf_index += 1;

        snapshot
    }
}
