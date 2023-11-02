use crate::abis::previous_rollup_data::PreviousRollupData;
use crate::abis::base_or_merge_rollup_public_inputs::{BaseOrMergeRollupPublicInputs, MERGE_ROLLUP_TYPE};
use crate::components;

struct MergeRollupInputs{
    // TODO(Kev): Why is this 2?
    previous_rollup_data : [PreviousRollupData; 2]
}

impl MergeRollupInputs {
    pub fn merge_rollup_circuit(self) -> BaseOrMergeRollupPublicInputs {

        // TODO(Lasse): Verify the previous rollup proofs
        // TODO(Lasse): Check both previous rollup vks (in previous_rollup_data) against the permitted set of kernel vks.
        // we don't have a set of permitted kernel vks yet.

        let left = self.previous_rollup_data[0].base_or_merge_rollup_public_inputs;
        let right = self.previous_rollup_data[1].base_or_merge_rollup_public_inputs;

        // check that both input proofs are either both "BASE" or "MERGE" and not a mix!
        // this prevents having wonky commitment, nullifier and contract subtrees.
        let aggregation_object = components::aggregate_proofs(left, right);
        components::assert_both_input_proofs_of_same_rollup_type(left, right);
        let current_height = components::assert_both_input_proofs_of_same_height_and_return(left, right);
        components::assert_equal_constants(left, right);
        components::assert_prev_rollups_follow_on_from_each_other(left, right);

        // compute calldata hash:
        let new_calldata_hash = components::compute_calldata_hash(self.previous_rollup_data);

        let public_inputs = BaseOrMergeRollupPublicInputs {
            rollup_type : MERGE_ROLLUP_TYPE,
            rollup_subtree_height : current_height + 1,
            end_aggregation_object : aggregation_object,
            constants : left.constants,
            start_note_hash_tree_snapshot : left.start_note_hash_tree_snapshot,
            end_note_hash_tree_snapshot : right.end_note_hash_tree_snapshot,
            start_nullifier_tree_snapshot : left.start_nullifier_tree_snapshot,
            end_nullifier_tree_snapshot : right.end_nullifier_tree_snapshot,
            start_contract_tree_snapshot : left.start_contract_tree_snapshot,
            end_contract_tree_snapshot : right.end_contract_tree_snapshot,
            start_public_data_tree_root : left.start_public_data_tree_root,
            end_public_data_tree_root : right.end_public_data_tree_root,
            calldata_hash : new_calldata_hash,
        };

        public_inputs
    }
}