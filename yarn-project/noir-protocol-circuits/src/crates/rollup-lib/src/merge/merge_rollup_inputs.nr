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
            aggregation_object : aggregation_object,
            constants : left.constants,
            start : left.start,
            end : right.end,
            calldata_hash : new_calldata_hash,
        };

        public_inputs
    }
}

mod tests {
    use crate::{
        merge::merge_rollup_inputs::MergeRollupInputs,
        tests::merge_rollup_inputs::default_merge_rollup_inputs,
    };
    use dep::types::hash::accumulate_sha256;

    #[test(should_fail_with="input proofs are of different rollup types")]
    fn different_rollup_type_fails() {
        let mut inputs = default_merge_rollup_inputs();
        inputs.previous_rollup_data[0].base_or_merge_rollup_public_inputs.rollup_type = 0;
        inputs.previous_rollup_data[1].base_or_merge_rollup_public_inputs.rollup_type = 1;
        let _output = inputs.merge_rollup_circuit();
    }

    #[test(should_fail_with="input proofs are of different rollup heights")]
    fn different_height_fails() {
        let mut inputs = default_merge_rollup_inputs();
        inputs.previous_rollup_data[0].base_or_merge_rollup_public_inputs.rollup_subtree_height = 0;
        inputs.previous_rollup_data[1].base_or_merge_rollup_public_inputs.rollup_subtree_height = 1;
        let _output = inputs.merge_rollup_circuit();
    }

    #[test(should_fail_with="input proofs have different constants")]
    fn constants_different_fails() {
        let mut inputs = default_merge_rollup_inputs();
        inputs.previous_rollup_data[0].base_or_merge_rollup_public_inputs.constants.public_kernel_vk_tree_root = 1;
        inputs.previous_rollup_data[1].base_or_merge_rollup_public_inputs.constants.public_kernel_vk_tree_root = 0;
        let _output = inputs.merge_rollup_circuit();
    }

    #[test(should_fail_with="input proofs have different constants")]
    fn constants_different_chain_id_fails() {
        let mut inputs = default_merge_rollup_inputs();
        inputs.previous_rollup_data[0].base_or_merge_rollup_public_inputs.constants.global_variables.chain_id = 1;
        inputs.previous_rollup_data[1].base_or_merge_rollup_public_inputs.constants.global_variables.chain_id = 0;
        let _output = inputs.merge_rollup_circuit();
    }

    #[test(should_fail_with="input proofs have different note hash tree snapshots")]
    fn previous_rollups_dont_follow_note_hash() {
        let mut inputs = default_merge_rollup_inputs();
        inputs.previous_rollup_data[0].base_or_merge_rollup_public_inputs.end.note_hash_tree.root = 0;
        inputs.previous_rollup_data[1].base_or_merge_rollup_public_inputs.start.note_hash_tree.root = 1;
        let _output = inputs.merge_rollup_circuit();
    }

    #[test(should_fail_with="input proofs have different nullifier tree snapshots")]
    fn previous_rollups_dont_follow_nullifier() {
        let mut inputs = default_merge_rollup_inputs();
        inputs.previous_rollup_data[0].base_or_merge_rollup_public_inputs.end.nullifier_tree.root = 0;
        inputs.previous_rollup_data[1].base_or_merge_rollup_public_inputs.start.nullifier_tree.root = 1;
        let _output = inputs.merge_rollup_circuit();
    }

    #[test(should_fail_with="input proofs have different contract tree snapshots")]
    fn previous_rollups_dont_follow_contracts() {
        let mut inputs = default_merge_rollup_inputs();
        inputs.previous_rollup_data[0].base_or_merge_rollup_public_inputs.end.contract_tree.root = 0;
        inputs.previous_rollup_data[1].base_or_merge_rollup_public_inputs.start.contract_tree.root = 1;
        let _output = inputs.merge_rollup_circuit();
    }

    #[test]
    fn rollup_fields_are_set_correctly() {
        let mut inputs = default_merge_rollup_inputs();
        let mut outputs = inputs.merge_rollup_circuit();
        assert_eq(outputs.rollup_type, 1);
        assert_eq(
            outputs.rollup_subtree_height, inputs.previous_rollup_data[0].base_or_merge_rollup_public_inputs.rollup_subtree_height + 1
        );

        // set inputs to have a merge rollup type and set the rollup height and test again.
        inputs.previous_rollup_data[0].base_or_merge_rollup_public_inputs.rollup_type = 1;
        inputs.previous_rollup_data[0].base_or_merge_rollup_public_inputs.rollup_subtree_height = 1;

        inputs.previous_rollup_data[1].base_or_merge_rollup_public_inputs.rollup_type = 1;
        inputs.previous_rollup_data[1].base_or_merge_rollup_public_inputs.rollup_subtree_height = 1;

        outputs = inputs.merge_rollup_circuit();
        assert_eq(outputs.rollup_type, 1);
        assert_eq(outputs.rollup_subtree_height, 2);
    }

    #[test]
    fn start_and_end_partial_states() {
        let mut inputs = default_merge_rollup_inputs();
        let outputs = inputs.merge_rollup_circuit();

        assert(outputs.start.eq(inputs.previous_rollup_data[0].base_or_merge_rollup_public_inputs.start));
        assert(outputs.end.eq(inputs.previous_rollup_data[1].base_or_merge_rollup_public_inputs.end));
    }

    #[test]
    fn calldata_hash() {
        let mut inputs = default_merge_rollup_inputs();
        let expected_calldata_hash = accumulate_sha256(
            [
            U128::from_integer(0),
            U128::from_integer(1),
            U128::from_integer(2),
            U128::from_integer(3)
        ]
        );
        let outputs = inputs.merge_rollup_circuit();

        assert_eq(outputs.calldata_hash, expected_calldata_hash);
    }

    #[test]
    fn constants_dont_change() {
        let mut inputs = default_merge_rollup_inputs();
        let outputs = inputs.merge_rollup_circuit();

        assert(
            inputs.previous_rollup_data[0].base_or_merge_rollup_public_inputs.constants.eq(outputs.constants)
        );
        assert(
            inputs.previous_rollup_data[1].base_or_merge_rollup_public_inputs.constants.eq(outputs.constants)
        );
    }
}
