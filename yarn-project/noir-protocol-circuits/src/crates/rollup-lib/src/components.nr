use crate::abis::base_or_merge_rollup_public_inputs::{BaseOrMergeRollupPublicInputs, AggregationObject};
use dep::private_kernel_lib::hash::accumulate_sha256;
use dep::private_kernel_lib::utils::uint128::U128;
use dep::aztec::constants_gen::NUM_FIELDS_PER_SHA256;
use crate::abis::previous_rollup_data::PreviousRollupData;

/**
 * Create an aggregation object for the proofs that are provided
 *          - We add points P0 for each of our proofs
 *          - We add points P1 for each of our proofs
 *          - We concat our public inputs
 * TODO(Kev): This seems similar to the aggregate_proof method in the private-kernel-lib
 */
pub fn aggregate_proofs(left : BaseOrMergeRollupPublicInputs, right : BaseOrMergeRollupPublicInputs) -> AggregationObject {
    // TODO: Similar to cpp code this does not do anything.
    left.end_aggregation_object
}

/**
 * Asserts that the rollup types are the same. 
 * Either both merge or both base
 */
pub fn assert_both_input_proofs_of_same_rollup_type(left : BaseOrMergeRollupPublicInputs, right : BaseOrMergeRollupPublicInputs)
{
    assert(left.rollup_type == right.rollup_type, "input proofs are of different rollup types");
}

/**
 * Asserts that the rollup subtree heights are the same and returns the height
 * Returns the height of the rollup subtrees
 */
pub fn assert_both_input_proofs_of_same_height_and_return(left : BaseOrMergeRollupPublicInputs, right : BaseOrMergeRollupPublicInputs) -> Field{
    assert(left.rollup_subtree_height == right.rollup_subtree_height, "input proofs are of different rollup heights");
    left.rollup_subtree_height
}

/**
 * Asserts that the constants used in the left and right child are identical
 *
 */
pub fn assert_equal_constants(left : BaseOrMergeRollupPublicInputs, right : BaseOrMergeRollupPublicInputs) {
    assert(left.constants.eq(right.constants), "input proofs have different constants");
}

// asserts that the end snapshot of previous_rollup 0 equals the start snapshot of previous_rollup 1 (i.e. ensure they
// follow on from one-another). Ensures that right uses the tres that was updated by left.
pub fn assert_prev_rollups_follow_on_from_each_other(left : BaseOrMergeRollupPublicInputs, right : BaseOrMergeRollupPublicInputs)
{
    assert(left.end_note_hash_tree_snapshot.eq(right.start_note_hash_tree_snapshot), "input proofs have different note hash tree snapshots");
    assert(left.end_nullifier_tree_snapshot.eq(right.start_nullifier_tree_snapshot),"input proofs have different nullifier tree snapshots");
    assert(left.end_contract_tree_snapshot.eq(right.start_contract_tree_snapshot), "input proofs have different contract tree snapshots");
    assert(left.end_public_data_tree_root == right.start_public_data_tree_root, "input proofs have different public data tree snapshots");
}

/**
 * @brief From two previous rollup data, compute a single calldata hash
 *
 * @param previous_rollup_data
 * @return calldata hash stored in 2 fields
 */
pub fn compute_calldata_hash(previous_rollup_data : [PreviousRollupData ; 2]) -> [Field; NUM_FIELDS_PER_SHA256]{
    accumulate_sha256([
        U128::from_field(previous_rollup_data[0].base_or_merge_rollup_public_inputs.calldata_hash[0]),
        U128::from_field(previous_rollup_data[0].base_or_merge_rollup_public_inputs.calldata_hash[1]),
        U128::from_field(previous_rollup_data[1].base_or_merge_rollup_public_inputs.calldata_hash[0]),
        U128::from_field(previous_rollup_data[1].base_or_merge_rollup_public_inputs.calldata_hash[1])
    ])
}
