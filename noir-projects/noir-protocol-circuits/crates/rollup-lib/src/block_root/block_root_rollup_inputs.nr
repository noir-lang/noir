use crate::{
    abis::{
    previous_rollup_data::PreviousRollupData, constant_rollup_data::ConstantRollupData,
    block_root_or_block_merge_public_inputs::{BlockRootOrBlockMergePublicInputs, FeeRecipient}
},
    components
};
use parity_lib::{root::root_rollup_parity_input::RootRollupParityInput, ParityPublicInputs};
use types::{
    abis::{append_only_tree_snapshot::AppendOnlyTreeSnapshot, nullifier_leaf_preimage::NullifierLeafPreimage},
    address::EthAddress,
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

struct BlockRootRollupInputs {
    // All below are shared between the base and merge rollups
    previous_rollup_data: [PreviousRollupData; 2],

    l1_to_l2_roots: RootRollupParityInput,

    // inputs required to process l1 to l2 messages
    l1_to_l2_messages: [Field; NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP],
    l1_to_l2_message_subtree_sibling_path: [Field; L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH],

    start_l1_to_l2_message_tree_snapshot: AppendOnlyTreeSnapshot,

    // inputs required to add the block hash
    start_archive_snapshot: AppendOnlyTreeSnapshot,
    new_archive_sibling_path: [Field; ARCHIVE_HEIGHT],
    // Added previous_block_hash to be passed through to the final root, where it will be either:
    // - checked on L1 for first block in batch
    // - checked against previous block_root.end_block_hash in a block_merge
    previous_block_hash: Field,
    // TODO(#7346): Temporarily added prover_id while we verify block-root proofs on L1
    prover_id: Field,
}

impl BlockRootRollupInputs {
    pub fn block_root_rollup_circuit(self) -> BlockRootOrBlockMergePublicInputs {
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

        let mut fee_arr = [FeeRecipient::empty(); 32];
        fee_arr[0] = FeeRecipient { recipient: left.constants.global_variables.coinbase, value: total_fees };

        BlockRootOrBlockMergePublicInputs {
            previous_archive: left.constants.last_archive, // archive before this block was added
            new_archive: archive, // archive once this block was added
            previous_block_hash: self.previous_block_hash,
            end_block_hash: block_hash, // current newest block hash = this block hash
            start_global_variables: left.constants.global_variables, // we have asserted that left.constants == right.constants => ...
            end_global_variables: left.constants.global_variables, // with a current block range of 1, we only have 1 set of constants
            out_hash: content_commitment.out_hash,
            fees: fee_arr,
            vk_tree_root: left.constants.vk_tree_root,
            prover_id: self.prover_id
        }
    }
}

impl Empty for BlockRootRollupInputs {
    fn empty() -> Self {
        BlockRootRollupInputs {
            previous_rollup_data: [PreviousRollupData::empty(); 2],
            l1_to_l2_roots: RootRollupParityInput::empty(),
            l1_to_l2_messages: [0; NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP],
            l1_to_l2_message_subtree_sibling_path: [0; L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH],
            start_l1_to_l2_message_tree_snapshot: AppendOnlyTreeSnapshot::zero(),
            start_archive_snapshot: AppendOnlyTreeSnapshot::zero(),
            new_archive_sibling_path: [0; ARCHIVE_HEIGHT],
            previous_block_hash: 0,
            prover_id: 0,
        }
    }
}
