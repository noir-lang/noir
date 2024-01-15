use crate::abis::base_or_merge_rollup_public_inputs::BaseOrMergeRollupPublicInputs;
use dep::types::mocked::AggregationObject;
use dep::types::hash::{accumulate_sha256, assert_check_membership, root_from_sibling_path};
use dep::types::constants::NUM_FIELDS_PER_SHA256;
use crate::abis::previous_rollup_data::PreviousRollupData;
use dep::types::abis::append_only_tree_snapshot::AppendOnlyTreeSnapshot;

/**
 * Create an aggregation object for the proofs that are provided
 *          - We add points P0 for each of our proofs
 *          - We add points P1 for each of our proofs
 *          - We concat our public inputs
 * TODO(Kev): This seems similar to the aggregate_proof method in the private-kernel-lib
 */
pub fn aggregate_proofs(
    left: BaseOrMergeRollupPublicInputs,
    right: BaseOrMergeRollupPublicInputs
) -> AggregationObject {
    // TODO: Similar to cpp code this does not do anything.
    left.aggregation_object
}

/**
 * Asserts that the rollup types are the same. 
 * Either both merge or both base
 */
pub fn assert_both_input_proofs_of_same_rollup_type(
    left: BaseOrMergeRollupPublicInputs,
    right: BaseOrMergeRollupPublicInputs
) {
    assert(left.rollup_type == right.rollup_type, "input proofs are of different rollup types");
}

/**
 * Asserts that the rollup subtree heights are the same and returns the height
 * Returns the height of the rollup subtrees
 */
pub fn assert_both_input_proofs_of_same_height_and_return(
    left: BaseOrMergeRollupPublicInputs,
    right: BaseOrMergeRollupPublicInputs
) -> Field {
    assert(
        left.rollup_subtree_height == right.rollup_subtree_height, "input proofs are of different rollup heights"
    );
    left.rollup_subtree_height
}

/**
 * Asserts that the constants used in the left and right child are identical
 *
 */
pub fn assert_equal_constants(
    left: BaseOrMergeRollupPublicInputs,
    right: BaseOrMergeRollupPublicInputs
) {
    assert(left.constants.eq(right.constants), "input proofs have different constants");
}

// asserts that the end snapshot of previous_rollup 0 equals the start snapshot of previous_rollup 1 (i.e. ensure they
// follow on from one-another). Ensures that right uses the tres that was updated by left.
pub fn assert_prev_rollups_follow_on_from_each_other(
    left: BaseOrMergeRollupPublicInputs,
    right: BaseOrMergeRollupPublicInputs
) {
    assert(
        left.end.note_hash_tree.eq(right.start.note_hash_tree), "input proofs have different note hash tree snapshots"
    );
    assert(
        left.end.nullifier_tree.eq(right.start.nullifier_tree), "input proofs have different nullifier tree snapshots"
    );
    assert(
        left.end.contract_tree.eq(right.start.contract_tree), "input proofs have different contract tree snapshots"
    );
    assert(
        left.end.public_data_tree.eq(right.start.public_data_tree), "input proofs have different public data tree snapshots"
    );
}

/**
 * @brief From two previous rollup data, compute a single calldata hash
 *
 * @param previous_rollup_data
 * @return calldata hash stored in 2 fields
 */
pub fn compute_calldata_hash(previous_rollup_data: [PreviousRollupData; 2]) -> [Field; NUM_FIELDS_PER_SHA256] {
    accumulate_sha256(
        [
        U128::from_integer(previous_rollup_data[0].base_or_merge_rollup_public_inputs.calldata_hash[0]),
        U128::from_integer(previous_rollup_data[0].base_or_merge_rollup_public_inputs.calldata_hash[1]),
        U128::from_integer(previous_rollup_data[1].base_or_merge_rollup_public_inputs.calldata_hash[0]),
        U128::from_integer(previous_rollup_data[1].base_or_merge_rollup_public_inputs.calldata_hash[1])
    ]
    )
}

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
