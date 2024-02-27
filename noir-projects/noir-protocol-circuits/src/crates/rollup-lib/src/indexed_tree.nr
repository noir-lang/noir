use crate::merkle_tree::{calculate_subtree, calculate_empty_tree_root};

use dep::types::abis::{append_only_tree_snapshot::AppendOnlyTreeSnapshot, membership_witness::MembershipWitness};

fn check_permutation<T, N>(
    original_array: [T; N],
    sorted_array: [T; N],
    indexes: [u64; N],
    is_equal: fn(T, T) -> bool
) {
    let mut seen_value = [false; N];
    for i in 0..N {
        let index = indexes[i];
        let sorted_value = sorted_array[i];
        let original_value = original_array[index];
        assert(is_equal(sorted_value, original_value), "Invalid index");
        assert(!seen_value[index], "Duplicated index");
        seen_value[index] = true;
    }
}

#[test]
fn check_permutation_basic_test() {
    let original_array = [1, 2, 3];
    let sorted_array = [3, 1, 2];
    let indexes = [2, 0, 1];
    let is_equal = |a: Field, b: Field| a == b;
    check_permutation(original_array, sorted_array, indexes, is_equal);
}

#[test(should_fail_with = "Duplicated index")]
fn check_permutation_duplicated_index() {
    let original_array = [0, 1, 0];
    let sorted_array = [1, 0, 0];
    let indexes = [1, 0, 0];
    let is_equal = |a: Field, b: Field| a == b;
    check_permutation(original_array, sorted_array, indexes, is_equal);
}

#[test(should_fail_with = "Invalid index")]
fn check_permutation_invalid_index() {
    let original_array = [0, 1, 2];
    let sorted_array = [1, 0, 0];
    let indexes = [1, 0, 2];
    let is_equal = |a: Field, b: Field| a == b;
    check_permutation(original_array, sorted_array, indexes, is_equal);
}

pub fn batch_insert<Value, Leaf, SubtreeWidth, SiblingPathLength, SubtreeHeight, TreeHeight>(
    start_snapshot: AppendOnlyTreeSnapshot,
    values_to_insert: [Value; SubtreeWidth],
    sorted_values: [Value; SubtreeWidth],
    sorted_values_indexes: [u64; SubtreeWidth],
    new_subtree_sibling_path: [Field; SiblingPathLength],
    low_leaf_preimages: [Leaf; SubtreeWidth],
    low_leaf_membership_witnesses: [MembershipWitness<TreeHeight>; SubtreeWidth],
    is_equal: fn(Value, Value) -> bool,
    is_empty_value: fn(Value) -> bool,
    hash_leaf: fn(Leaf) -> Field,
    is_valid_low_leaf: fn(Leaf, Value) -> bool,
    update_low_leaf: fn(Leaf, Value, u64) -> Leaf,
    build_insertion_leaf: fn(Value, Leaf) -> Leaf,
    _subtree_height: [Field; SubtreeHeight],
    _tree_height: [Field; TreeHeight]
) -> AppendOnlyTreeSnapshot {
    // A permutation to the values is provided to make the insertion use only one insertion strategy
    check_permutation(
        values_to_insert,
        sorted_values,
        sorted_values_indexes,
        is_equal
    );

    // Now, update the existing leaves with the new leaves
    let mut current_tree_root = start_snapshot.root;
    let mut insertion_subtree: [Leaf; SubtreeWidth] = dep::std::unsafe::zeroed();
    let start_insertion_index = start_snapshot.next_available_leaf_index;

    for i in 0..sorted_values.len() {
        let value = sorted_values[i];
        if !is_empty_value(value) {
            let low_leaf_preimage = low_leaf_preimages[i];
            let witness = low_leaf_membership_witnesses[i];

            assert(is_valid_low_leaf(low_leaf_preimage, value), "Invalid low leaf");

            // perform membership check for the low leaf against the original root
            crate::components::assert_check_membership(
                hash_leaf(low_leaf_preimage),
                witness.leaf_index,
                witness.sibling_path,
                current_tree_root
            );

            let value_index = sorted_values_indexes[i];

            // Calculate the new value of the low_leaf
            let updated_low_leaf = update_low_leaf(
                low_leaf_preimage,
                value,
                start_insertion_index as u64 + value_index
            );

            current_tree_root = crate::components::root_from_sibling_path(
                hash_leaf(updated_low_leaf), witness.leaf_index, witness.sibling_path);

            insertion_subtree[value_index] = build_insertion_leaf(value, low_leaf_preimage);
        }
    }

    let empty_subtree_root = calculate_empty_tree_root(SubtreeHeight);
    let leaf_index_subtree_depth = start_insertion_index >> (SubtreeHeight as u32);

    crate::components::assert_check_membership(
        empty_subtree_root,
        leaf_index_subtree_depth as Field,
        new_subtree_sibling_path,
        current_tree_root
    );

    // Create new subtree to insert into the whole indexed tree
    let subtree_root = calculate_subtree(insertion_subtree.map(hash_leaf));

    // Calculate the new root
    // We are inserting a subtree rather than a full tree here
    let subtree_index = start_insertion_index >> (SubtreeHeight as u32);
    let new_root = crate::components::root_from_sibling_path(subtree_root, subtree_index as Field, new_subtree_sibling_path);

    AppendOnlyTreeSnapshot { root: new_root, next_available_leaf_index: start_insertion_index + (values_to_insert.len() as u32) }
}

