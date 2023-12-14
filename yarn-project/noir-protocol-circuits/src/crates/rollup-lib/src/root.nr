mod root_rollup_inputs;
use root_rollup_inputs::RootRollupInputs;
mod root_rollup_public_inputs;
use root_rollup_public_inputs::RootRollupPublicInputs;
use crate::abis::append_only_tree_snapshot::AppendOnlyTreeSnapshot;
use dep::types::utils::uint256::U256;
use dep::types::constants::{NUM_FIELDS_PER_SHA256,NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,L1_TO_L2_MSG_SUBTREE_HEIGHT};
use crate::{components, hash::compute_block_hash_with_globals};
use crate::merkle_tree::{calculate_subtree, calculate_empty_tree_root};

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
        let new_l1_to_l2_messages_tree_snapshot = components::insert_subtree_to_snapshot_tree(
            self.start_l1_to_l2_messages_tree_snapshot,
            self.new_l1_to_l2_messages_tree_root_sibling_path,
            empty_l1_to_l2_subtree_root,
            l1_to_l2_subtree_root,
            // TODO(Kev): For now we can add a test that this fits inside of 
            // a u8.
            L1_TO_L2_MSG_SUBTREE_HEIGHT as u8 
        );

        // Build the block hash for this iteration from the tree roots and global variables
        // Then insert the block into the archive tree
        let block_hash = compute_block_hash_with_globals(left.constants.global_variables,
                                                      right.end_note_hash_tree_snapshot.root,
                                                      right.end_nullifier_tree_snapshot.root,
                                                      right.end_contract_tree_snapshot.root,
                                                      new_l1_to_l2_messages_tree_snapshot.root,
                                                      right.end_public_data_tree_snapshot.root);

        // Update the archive
        let end_archive_snapshot = components::insert_subtree_to_snapshot_tree(
            self.start_archive_snapshot,
            self.new_archive_sibling_path,
            0,
            block_hash,
            0
        );

        let zeroed_out_snapshot = AppendOnlyTreeSnapshot {
            root : 0,
            next_available_leaf_index : 0
        };

        RootRollupPublicInputs{
            end_aggregation_object : aggregation_object,
            global_variables : left.constants.global_variables,
            start_note_hash_tree_snapshot : left.start_note_hash_tree_snapshot,
            end_note_hash_tree_snapshot : right.end_note_hash_tree_snapshot,
            start_nullifier_tree_snapshot : left.start_nullifier_tree_snapshot,
            end_nullifier_tree_snapshot : right.end_nullifier_tree_snapshot,
            start_contract_tree_snapshot : left.start_contract_tree_snapshot,
            end_contract_tree_snapshot : right.end_contract_tree_snapshot,
            start_public_data_tree_snapshot : left.start_public_data_tree_snapshot,
            end_public_data_tree_snapshot : right.end_public_data_tree_snapshot,
            start_l1_to_l2_messages_tree_snapshot : self.start_l1_to_l2_messages_tree_snapshot,
            end_l1_to_l2_messages_tree_snapshot : new_l1_to_l2_messages_tree_snapshot,
            start_archive_snapshot : self.start_archive_snapshot,
            end_archive_snapshot : end_archive_snapshot,
            calldata_hash : components::compute_calldata_hash(self.previous_rollup_data),
            l1_to_l2_messages_hash : compute_messages_hash(self.new_l1_to_l2_messages),
        }
    }
}

// See `test_message_input_flattened_length` on keeping this in sync,
// why its here and how this constant was computed.
global NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP_NUM_BYTES = 512;

// Computes the messages hash from the leaves array
//
// Returns the hash split into two field elements
fn compute_messages_hash(leaves : [Field; NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP]) -> [Field;NUM_FIELDS_PER_SHA256] {
    
    // Slice variation
    // let mut hash_input_flattened = [];
    // for leaf in leaves {
    //     let input_as_bytes = leaf.to_be_bytes(32);
    //     for i in 0..32 {
    //         // TODO(Kev): should check the complexity of repeatedly pushing
    //         hash_input_flattened.push(input_as_bytes[i]);
    //     }
    // }

    // Convert each field element into a byte array and append the bytes to `hash_input_flattened`
    let mut hash_input_flattened = [0; NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP_NUM_BYTES];
    for offset in 0..NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP {
        let input_as_bytes = leaves[offset].to_be_bytes(32);
        for byte_index in 0..32 {
            hash_input_flattened[offset * 32 + byte_index] = input_as_bytes[byte_index];
        }
    }

    // Hash bytes and convert to 2 128 bit limbs
    let sha_digest = dep::std::hash::sha256(hash_input_flattened);
    // TODO(Kev): The CPP implementation is returning [high, low]
    // and so is `to_u128_limbs`, so this matches.
    // We should say why we are doing this vs [low, high]
    U256::from_bytes32(sha_digest).to_u128_limbs()
}

#[test]
fn test_message_input_flattened_length() {
    // This is here so that the global doesn't become outdated.
    // 
    // The short term solution to remove this is to use slices, though
    // those are a bit experimental right now, so TODO I'll add a test that the
    // slice version of compute_messages_hash is the same as the array version.
    // which uses the NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP_NUM_BYTES global.
    assert(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP * 32 == NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP_NUM_BYTES);
}


mod tests {
    use crate::{
        root::{
            root_rollup_inputs::RootRollupInputs,
            NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP_NUM_BYTES,
        },
        tests::root_rollup_inputs::default_root_rollup_inputs,
    };
    use dep::types::utils::uint128::U128;
    use dep::types::utils::uint256::U256;
    use dep::types::hash::accumulate_sha256;


    #[test]
    fn check_block_hashes_empty_blocks(){
        let expected_messages_hash = U256::from_bytes32(dep::std::hash::sha256([0; NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP_NUM_BYTES])).to_u128_limbs();

        let expected_calldata_hash = accumulate_sha256([
            U128::from_field(0),
            U128::from_field(1),
            U128::from_field(2),
            U128::from_field(3)
        ]);

        let inputs = default_root_rollup_inputs();
        let outputs = inputs.root_rollup_circuit();
    
        // check calldata hash
        assert_eq(outputs.calldata_hash, expected_calldata_hash);
        // Check messages hash
        assert_eq(outputs.l1_to_l2_messages_hash, expected_messages_hash);
    }

    #[test]
    fn start_and_end_snapshots(){
        let inputs = default_root_rollup_inputs();
        let outputs = inputs.root_rollup_circuit();
        
        assert(
            outputs.start_note_hash_tree_snapshot.eq(inputs.previous_rollup_data[0].base_or_merge_rollup_public_inputs.start_note_hash_tree_snapshot)
        );
        assert(
            outputs.end_note_hash_tree_snapshot.eq(inputs.previous_rollup_data[1].base_or_merge_rollup_public_inputs.end_note_hash_tree_snapshot)
        );

        assert(
            outputs.start_nullifier_tree_snapshot.eq(inputs.previous_rollup_data[0].base_or_merge_rollup_public_inputs.start_nullifier_tree_snapshot)
        );
        assert(
            outputs.end_nullifier_tree_snapshot.eq(inputs.previous_rollup_data[1].base_or_merge_rollup_public_inputs.end_nullifier_tree_snapshot)
        );

        assert(
            outputs.start_contract_tree_snapshot.eq(inputs.previous_rollup_data[0].base_or_merge_rollup_public_inputs.start_contract_tree_snapshot)
        );
        assert(
            outputs.end_contract_tree_snapshot.eq(inputs.previous_rollup_data[1].base_or_merge_rollup_public_inputs.end_contract_tree_snapshot)
        );

        assert(
            outputs.start_public_data_tree_snapshot.eq(inputs.previous_rollup_data[0].base_or_merge_rollup_public_inputs.start_public_data_tree_snapshot)
        );
        assert(
            outputs.end_public_data_tree_snapshot.eq(inputs.previous_rollup_data[1].base_or_merge_rollup_public_inputs.end_public_data_tree_snapshot)
        );
    }

}