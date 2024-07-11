use dep::types::{traits::Empty, constants::{BASE_ROLLUP_INDEX, MERGE_ROLLUP_INDEX}};
use crate::abis::previous_rollup_data::PreviousRollupData;
use crate::abis::base_or_merge_rollup_public_inputs::{BaseOrMergeRollupPublicInputs, MERGE_ROLLUP_TYPE};
use crate::components;

global ALLOWED_PREVIOUS_CIRCUITS = [
    BASE_ROLLUP_INDEX,
    MERGE_ROLLUP_INDEX,
];

struct MergeRollupInputs {
    // TODO(Kev): Why is this 2?
    previous_rollup_data : [PreviousRollupData; 2]
}

impl Empty for MergeRollupInputs {
    fn empty() -> Self {
        MergeRollupInputs {
            previous_rollup_data: [PreviousRollupData::empty(); 2]
        }
    }
}

impl MergeRollupInputs {
    pub fn merge_rollup_circuit(self) -> BaseOrMergeRollupPublicInputs {
        // TODO(Lasse): Check both previous rollup vks (in previous_rollup_data) against the permitted set of kernel vks.
        // we don't have a set of permitted kernel vks yet.

        // Verify the previous rollup proofs
        if !dep::std::runtime::is_unconstrained() {
            self.previous_rollup_data[0].verify();
            // TODO(#7410) we need the tube vk to reinstate this
            // self.previous_rollup_data[0].validate_in_vk_tree(ALLOWED_PREVIOUS_CIRCUITS);

            self.previous_rollup_data[1].verify();
            // TODO(#7410) we need the tube vk to reinstate this
            // self.previous_rollup_data[1].validate_in_vk_tree(ALLOWED_PREVIOUS_CIRCUITS);
        }

        let left = self.previous_rollup_data[0].base_or_merge_rollup_public_inputs;
        let right = self.previous_rollup_data[1].base_or_merge_rollup_public_inputs;

        components::assert_txs_filled_from_left(left, right);
        components::assert_equal_constants(left, right);
        components::assert_prev_rollups_follow_on_from_each_other(left, right);

        // compute calldata hash:
        let txs_effects_hash = components::compute_txs_effects_hash(self.previous_rollup_data);
        let out_hash = components::compute_out_hash(self.previous_rollup_data);

        let accumulated_fees = components::accumulate_fees(left, right);

        let public_inputs = BaseOrMergeRollupPublicInputs {
            rollup_type: MERGE_ROLLUP_TYPE,
            num_txs: left.num_txs + right.num_txs,
            constants: left.constants,
            start: left.start,
            end: right.end,
            txs_effects_hash,
            out_hash,
            accumulated_fees
        };

        public_inputs
    }
}

mod tests {
    use crate::{
        merge::merge_rollup_inputs::MergeRollupInputs,
        tests::merge_rollup_inputs::default_merge_rollup_inputs
    };
    use dep::types::hash::accumulate_sha256;
    use dep::types::constants::{ROOT_PARITY_INDEX, MERGE_ROLLUP_INDEX, BASE_ROLLUP_INDEX};

    #[test(should_fail_with="The rollup should be filled greedily from L to R, but received a L base and R merge")]
    fn different_rollup_type_fails() {
        let mut inputs = default_merge_rollup_inputs();
        inputs.previous_rollup_data[0].base_or_merge_rollup_public_inputs.rollup_type = 0;
        inputs.previous_rollup_data[1].base_or_merge_rollup_public_inputs.rollup_type = 1;
        let _output = inputs.merge_rollup_circuit();
    }

    #[test(should_fail_with="input proofs have different constants")]
    fn constants_different_fails() {
        let mut inputs = default_merge_rollup_inputs();
        inputs.previous_rollup_data[0].base_or_merge_rollup_public_inputs.constants.global_variables.chain_id = 1;
        inputs.previous_rollup_data[1].base_or_merge_rollup_public_inputs.constants.global_variables.chain_id = 0;
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

    #[test]
    fn rollup_fields_are_set_correctly() {
        let mut inputs = default_merge_rollup_inputs();
        let mut outputs = inputs.merge_rollup_circuit();
        assert_eq(outputs.rollup_type, 1);
        assert_eq(outputs.num_txs, 2);

        // set inputs to have a merge rollup type and set the rollup height and test again.
        inputs.previous_rollup_data[0].base_or_merge_rollup_public_inputs.rollup_type = 1;
        inputs.previous_rollup_data[0].base_or_merge_rollup_public_inputs.num_txs = 2;

        inputs.previous_rollup_data[1].base_or_merge_rollup_public_inputs.rollup_type = 1;
        inputs.previous_rollup_data[1].base_or_merge_rollup_public_inputs.num_txs = 2;

        outputs = inputs.merge_rollup_circuit();
        assert_eq(outputs.rollup_type, 1);
        assert_eq(outputs.num_txs, 4);
    }

    #[test]
    fn tx_subtrees_are_set_correctly() {
        let mut inputs = default_merge_rollup_inputs();
        let mut outputs = inputs.merge_rollup_circuit();
        // Start with two bases => two single txs
        assert_eq(outputs.num_txs, 2);
        // Test one merge of 2 merged with one base
        inputs.previous_rollup_data[0].base_or_merge_rollup_public_inputs.rollup_type = 1;
        inputs.previous_rollup_data[0].base_or_merge_rollup_public_inputs.num_txs = 2;
        outputs = inputs.merge_rollup_circuit();
        // Should have one subtree of size 2, and one of size 1
        assert_eq(outputs.num_txs, 3);
        // Test two merges, each with a subtree of 2
        inputs.previous_rollup_data[1].base_or_merge_rollup_public_inputs.rollup_type = 1;
        inputs.previous_rollup_data[1].base_or_merge_rollup_public_inputs.num_txs = 2;
        outputs = inputs.merge_rollup_circuit();
        // Should have one subtree of size 4
        assert_eq(outputs.num_txs, 4);

        // Test two merges, one with a subtree of 16, one with subtrees of 4 and 1
        inputs.previous_rollup_data[0].base_or_merge_rollup_public_inputs.num_txs = 16;
        inputs.previous_rollup_data[1].base_or_merge_rollup_public_inputs.num_txs = 5;
        outputs = inputs.merge_rollup_circuit();
        assert_eq(outputs.num_txs, 21);
    }

    #[test]
    fn start_and_end_partial_states() {
        let mut inputs = default_merge_rollup_inputs();
        let outputs = inputs.merge_rollup_circuit();

        assert(outputs.start.eq(inputs.previous_rollup_data[0].base_or_merge_rollup_public_inputs.start));
        assert(outputs.end.eq(inputs.previous_rollup_data[1].base_or_merge_rollup_public_inputs.end));
    }

    #[test]
    fn txs_effects_hash() {
        let mut inputs = default_merge_rollup_inputs();
        let expected_hash = accumulate_sha256([1, 2]);
        let outputs = inputs.merge_rollup_circuit();

        assert_eq(outputs.txs_effects_hash, expected_hash);
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

    #[test]
    fn valid_previous_circuit_base() {
        let mut inputs = default_merge_rollup_inputs();

        let vk_tree = dep::types::tests::fixtures::vk_tree::get_vk_merkle_tree();

        inputs.previous_rollup_data[0].vk.hash = vk_tree.leaves[BASE_ROLLUP_INDEX];
        inputs.previous_rollup_data[0].vk_witness.leaf_index = BASE_ROLLUP_INDEX as Field;
        inputs.previous_rollup_data[0].vk_witness.sibling_path = vk_tree.get_sibling_path(BASE_ROLLUP_INDEX);

        let _outputs = inputs.merge_rollup_circuit();
    }

    #[test]
    fn valid_previous_circuit_merge() {
        let mut inputs = default_merge_rollup_inputs();

        let vk_tree = dep::types::tests::fixtures::vk_tree::get_vk_merkle_tree();

        inputs.previous_rollup_data[0].vk.hash = vk_tree.leaves[MERGE_ROLLUP_INDEX];
        inputs.previous_rollup_data[0].vk_witness.leaf_index = MERGE_ROLLUP_INDEX as Field;
        inputs.previous_rollup_data[0].vk_witness.sibling_path = vk_tree.get_sibling_path(MERGE_ROLLUP_INDEX);

        let _outputs = inputs.merge_rollup_circuit();
    }
    // TODO(#7410) we need the tube vk to reinstate this
    // #[test(should_fail_with="Invalid vk index")]
    // fn invalid_previous_circuit() {
    //     let mut inputs = default_merge_rollup_inputs();
    //     let vk_tree = dep::types::tests::fixtures::vk_tree::get_vk_merkle_tree();
    //     inputs.previous_rollup_data[0].vk.hash = vk_tree.leaves[ROOT_PARITY_INDEX];
    //     inputs.previous_rollup_data[0].vk_witness.leaf_index = ROOT_PARITY_INDEX as Field;
    //     inputs.previous_rollup_data[0].vk_witness.sibling_path = vk_tree.get_sibling_path(ROOT_PARITY_INDEX);
    //     let _outputs = inputs.merge_rollup_circuit();
    // }
}
