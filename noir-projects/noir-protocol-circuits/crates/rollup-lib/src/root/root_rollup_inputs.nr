use crate::{
    abis::{previous_rollup_data::PreviousRollupData, constant_rollup_data::ConstantRollupData},
    components, root::{compute_messages_hash, root_rollup_public_inputs::RootRollupPublicInputs}
};
use dep::types::{
    abis::{append_only_tree_snapshot::AppendOnlyTreeSnapshot, nullifier_leaf_preimage::NullifierLeafPreimage},
    constants::{
    NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP, L1_TO_L2_MSG_SUBTREE_HEIGHT,
    L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH, ARCHIVE_HEIGHT
},
    header::Header, content_commitment::ContentCommitment,
    merkle_tree::{append_only_tree, calculate_subtree, calculate_empty_tree_root},
    state_reference::StateReference
};

struct RootRollupInputs {
    // All below are shared between the base and merge rollups
    previous_rollup_data : [PreviousRollupData; 2],

    // inputs required to process l1 to l2 messages
    new_l1_to_l2_messages : [Field; NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP],
    new_l1_to_l2_message_tree_root_sibling_path : [Field; L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH],

    start_l1_to_l2_message_tree_snapshot : AppendOnlyTreeSnapshot,

    // inputs required to add the block hash
    start_archive_snapshot : AppendOnlyTreeSnapshot,
    new_archive_sibling_path : [Field; ARCHIVE_HEIGHT],
}

impl RootRollupInputs {
    pub fn root_rollup_circuit(self) -> RootRollupPublicInputs {
        let left = self.previous_rollup_data[0].base_or_merge_rollup_public_inputs;
        let right = self.previous_rollup_data[1].base_or_merge_rollup_public_inputs;

        let aggregation_object = components::aggregate_proofs(left, right);
        components::assert_both_input_proofs_of_same_rollup_type(left, right);
        let _ = components::assert_both_input_proofs_of_same_height_and_return(left, right);
        components::assert_equal_constants(left, right);
        components::assert_prev_rollups_follow_on_from_each_other(left, right);

        // Check correct l1 to l2 tree given
        // Compute subtree inserting l1 to l2 messages
        let l1_to_l2_subtree_root = calculate_subtree(self.new_l1_to_l2_messages);

        // Insert subtree into the l1 to l2 data tree
        let empty_l1_to_l2_subtree_root = calculate_empty_tree_root(L1_TO_L2_MSG_SUBTREE_HEIGHT);
        let new_l1_to_l2_message_tree_snapshot = append_only_tree::insert_subtree_to_snapshot_tree(
            self.start_l1_to_l2_message_tree_snapshot,
            self.new_l1_to_l2_message_tree_root_sibling_path,
            empty_l1_to_l2_subtree_root,
            l1_to_l2_subtree_root,
            // TODO(Kev): For now we can add a test that this fits inside of 
            // a u8.
            L1_TO_L2_MSG_SUBTREE_HEIGHT as u8
        );

        let state = StateReference { l1_to_l2_message_tree: new_l1_to_l2_message_tree_snapshot, partial: right.end };

        // TODO: in_hash: #4633 and out_hash: #4561
        let content_commitment = ContentCommitment {
            tx_tree_height: right.height_in_block_tree + 1,
            txs_effects_hash: components::compute_txs_effects_hash(self.previous_rollup_data),
            in_hash: [0, 0],
            out_hash: components::compute_out_hash(self.previous_rollup_data)
        };

        let header = Header {
            last_archive: left.constants.last_archive,
            content_commitment,
            state,
            global_variables: left.constants.global_variables
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

        RootRollupPublicInputs {
            aggregation_object,
            archive,
            header,
            // TODO(#4492): update this when implementing the new message model
            l1_to_l2_messages_hash: compute_messages_hash(self.new_l1_to_l2_messages)
        }
    }
}
