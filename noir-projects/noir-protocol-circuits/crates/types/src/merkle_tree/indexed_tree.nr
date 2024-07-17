mod check_valid_low_leaf;

use crate::{
    abis::{append_only_tree_snapshot::AppendOnlyTreeSnapshot},
    merkle_tree::{
    membership::{assert_check_membership, MembershipWitness},
    root::{calculate_subtree_root, calculate_empty_tree_root, root_from_sibling_path}
},
    traits::{Empty, Hash, is_empty}, utils::arrays::check_permutation
};

pub fn batch_insert<
    Value,
    Leaf,
    let SubtreeWidth: u32,
    let SiblingPathLength: u32,
    let SubtreeHeight: u32,
    let TreeHeight: u32
>(
    start_snapshot: AppendOnlyTreeSnapshot,
    values_to_insert: [Value; SubtreeWidth],
    sorted_values: [Value; SubtreeWidth],
    sorted_values_indexes: [u32; SubtreeWidth],
    new_subtree_sibling_path: [Field; SiblingPathLength],
    low_leaf_preimages: [Leaf; SubtreeWidth],
    low_leaf_membership_witnesses: [MembershipWitness<TreeHeight>; SubtreeWidth],
    is_valid_low_leaf: fn(Leaf, Value) -> bool,
    update_low_leaf: fn(Leaf, Value, u32) -> Leaf,
    build_insertion_leaf: fn(Value, Leaf) -> Leaf,
    _subtree_height: [Field; SubtreeHeight],
    _tree_height: [Field; TreeHeight]
) -> AppendOnlyTreeSnapshot where Value: Eq + Empty, Leaf: Hash + Empty {
    // A permutation to the values is provided to make the insertion use only one insertion strategy
    check_permutation(values_to_insert, sorted_values, sorted_values_indexes);

    // Now, update the existing leaves with the new leaves
    let mut current_tree_root = start_snapshot.root;
    let mut insertion_subtree = [Leaf::empty(); SubtreeWidth];
    let start_insertion_index = start_snapshot.next_available_leaf_index;

    for i in 0..sorted_values.len() {
        let value = sorted_values[i];
        if !is_empty(value) {
            let low_leaf_preimage = low_leaf_preimages[i];
            let witness = low_leaf_membership_witnesses[i];

            assert(is_valid_low_leaf(low_leaf_preimage, value), "Invalid low leaf");

            // perform membership check for the low leaf against the original root
            assert_check_membership(
                low_leaf_preimage.hash(),
                witness.leaf_index,
                witness.sibling_path,
                current_tree_root
            );

            let value_index = sorted_values_indexes[i];

            // Calculate the new value of the low_leaf
            let updated_low_leaf = update_low_leaf(
                low_leaf_preimage,
                value,
                start_insertion_index as u32 + value_index
            );

            current_tree_root = root_from_sibling_path(
                updated_low_leaf.hash(), witness.leaf_index, witness.sibling_path);

            insertion_subtree[value_index] = build_insertion_leaf(value, low_leaf_preimage);
        }
    }

    let empty_subtree_root = calculate_empty_tree_root(SubtreeHeight);
    let leaf_index_subtree_depth = start_insertion_index >> (SubtreeHeight as u8);

    assert_check_membership(
        empty_subtree_root,
        leaf_index_subtree_depth as Field,
        new_subtree_sibling_path,
        current_tree_root
    );

    // Create new subtree to insert into the whole indexed tree
    let subtree_root = calculate_subtree_root(insertion_subtree.map(|leaf: Leaf| leaf.hash()));

    // Calculate the new root
    // We are inserting a subtree rather than a full tree here
    let subtree_index = start_insertion_index >> (SubtreeHeight as u8);
    let new_root = root_from_sibling_path(subtree_root, subtree_index as Field, new_subtree_sibling_path);

    AppendOnlyTreeSnapshot { root: new_root, next_available_leaf_index: start_insertion_index + (values_to_insert.len() as u32) }
}
