use crate::{
    abis::{previous_rollup_data::PreviousRollupData, constant_rollup_data::ConstantRollupData},
    components, root::{root_rollup_public_inputs::RootRollupPublicInputs}
};
use parity_lib::{root::root_rollup_parity_input::RootRollupParityInput, ParityPublicInputs};
use types::{
    abis::{append_only_tree_snapshot::AppendOnlyTreeSnapshot, nullifier_leaf_preimage::NullifierLeafPreimage},
    constants::{
    NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP, L1_TO_L2_MSG_SUBTREE_HEIGHT,
    L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH, ARCHIVE_HEIGHT, BASE_ROLLUP_INDEX, MERGE_ROLLUP_INDEX
},
    header::Header, content_commitment::ContentCommitment,
    merkle_tree::{append_only_tree, calculate_subtree_root, calculate_empty_tree_root},
    state_reference::StateReference, traits::Empty
};

global ALLOWED_PREVIOUS_CIRCUITS = [
    BASE_ROLLUP_INDEX,
    MERGE_ROLLUP_INDEX,
];

struct RootRollupInputs {
    // All below are shared between the base and merge rollups
    previous_rollup_data : [PreviousRollupData; 2],

    l1_to_l2_roots: RootRollupParityInput,

    // inputs required to process l1 to l2 messages
    l1_to_l2_messages : [Field; NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP],
    l1_to_l2_message_subtree_sibling_path : [Field; L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH],

    start_l1_to_l2_message_tree_snapshot : AppendOnlyTreeSnapshot,

    // inputs required to add the block hash
    start_archive_snapshot : AppendOnlyTreeSnapshot,
    new_archive_sibling_path : [Field; ARCHIVE_HEIGHT],
}

impl RootRollupInputs {
    pub fn root_rollup_circuit(self) -> RootRollupPublicInputs {
        // Verify the previous rollup proofs
        if !dep::std::runtime::is_unconstrained() {
            self.previous_rollup_data[0].verify();
            // TODO(#7410) we need the tube vk to reinstate this
            // self.previous_rollup_data[0].validate_in_vk_tree(ALLOWED_PREVIOUS_CIRCUITS);

            self.previous_rollup_data[1].verify();
            // TODO(#7410) we need the tube vk to reinstate this
            // self.previous_rollup_data[1].validate_in_vk_tree(ALLOWED_PREVIOUS_CIRCUITS);

            // verify the root parity
            self.l1_to_l2_roots.verify();
            // TODO(#7410) we need the tube vk to reinstate this
            // self.l1_to_l2_roots.validate_in_vk_tree();
        }

        let left = self.previous_rollup_data[0].base_or_merge_rollup_public_inputs;
        let right = self.previous_rollup_data[1].base_or_merge_rollup_public_inputs;

        components::assert_txs_filled_from_left(left, right);
        components::assert_equal_constants(left, right);
        components::assert_prev_rollups_follow_on_from_each_other(left, right);

        // Insert subtree into the l1 to l2 data tree
        let empty_l1_to_l2_subtree_root = calculate_empty_tree_root(L1_TO_L2_MSG_SUBTREE_HEIGHT);
        let new_l1_to_l2_message_tree_snapshot = append_only_tree::insert_subtree_to_snapshot_tree(
            self.start_l1_to_l2_message_tree_snapshot,
            self.l1_to_l2_message_subtree_sibling_path,
            empty_l1_to_l2_subtree_root,
            self.l1_to_l2_roots.public_inputs.converted_root,
            // TODO(Kev): For now we can add a test that this fits inside of
            // a u8.
            L1_TO_L2_MSG_SUBTREE_HEIGHT as u8
        );

        let state = StateReference { l1_to_l2_message_tree: new_l1_to_l2_message_tree_snapshot, partial: right.end };

        let content_commitment = ContentCommitment {
            num_txs: (left.num_txs + right.num_txs) as Field,
            txs_effects_hash: components::compute_txs_effects_hash(self.previous_rollup_data),
            in_hash: self.l1_to_l2_roots.public_inputs.sha_root,
            out_hash: components::compute_out_hash(self.previous_rollup_data)
        };

        let total_fees = components::accumulate_fees(left, right);

        let vk_tree_root = left.constants.vk_tree_root;

        let header = Header {
            last_archive: left.constants.last_archive,
            content_commitment,
            state,
            global_variables: left.constants.global_variables,
            total_fees
        };

        // Build the block hash for this by hashing the header and then insert the new leaf to archive tree.
        let block_hash = header.hash();

        // Update the archive
        let archive = append_only_tree::insert_subtree_to_snapshot_tree(
            self.start_archive_snapshot,
            self.new_archive_sibling_path,
            0,
            block_hash,
            0
        );

        RootRollupPublicInputs { archive, header, vk_tree_root }
    }
}

impl Empty for RootRollupInputs {
    fn empty() -> Self {
        RootRollupInputs {
            previous_rollup_data : [PreviousRollupData::empty(); 2],
            l1_to_l2_roots: RootRollupParityInput::empty(),
            l1_to_l2_messages : [0; NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP],
            l1_to_l2_message_subtree_sibling_path : [0; L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH],
            start_l1_to_l2_message_tree_snapshot : AppendOnlyTreeSnapshot::zero(),
            start_archive_snapshot : AppendOnlyTreeSnapshot::zero(),
            new_archive_sibling_path : [0; ARCHIVE_HEIGHT],
        }
    }
}
