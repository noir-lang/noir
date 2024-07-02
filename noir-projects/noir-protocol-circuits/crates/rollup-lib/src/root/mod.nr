mod root_rollup_inputs;
mod root_rollup_public_inputs;

// Re-exports
use root_rollup_inputs::RootRollupInputs;
use root_rollup_public_inputs::RootRollupPublicInputs;

// TODO: Move all the following code to different files
use dep::types::{constants::NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP, utils::uint256::U256, hash::sha256_to_field};

// See `test_message_input_flattened_length` on keeping this in sync,
// why its here and how this constant was computed.
global NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP_NUM_BYTES: u32 = 512;

// Computes the messages hash from the leaves array
//
// TODO(Miranda): remove? This appears to be unused
// Returns the hash truncated to one field element
fn compute_messages_hash(leaves: [Field; NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP]) -> Field {
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

    sha256_to_field(hash_input_flattened)
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
        root::{root_rollup_inputs::RootRollupInputs, NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP_NUM_BYTES},
        tests::root_rollup_inputs::default_root_rollup_inputs
    };
    use dep::types::utils::{uint256::U256, field::field_from_bytes_32_trunc};
    use dep::types::hash::accumulate_sha256;

    #[test]
    fn check_block_hashes_empty_blocks() {
        let expected_txs_effects_hash = accumulate_sha256([1, 2]);

        let inputs = default_root_rollup_inputs();
        let outputs = inputs.root_rollup_circuit();

        // check txs effects hash
        assert_eq(outputs.header.content_commitment.txs_effects_hash, expected_txs_effects_hash);
    }

    #[test]
    fn end_state() {
        let inputs = default_root_rollup_inputs();
        let outputs = inputs.root_rollup_circuit();

        assert(
            outputs.header.state.partial.note_hash_tree.eq(inputs.previous_rollup_data[1].base_or_merge_rollup_public_inputs.end.note_hash_tree)
        );

        assert(
            outputs.header.state.partial.nullifier_tree.eq(inputs.previous_rollup_data[1].base_or_merge_rollup_public_inputs.end.nullifier_tree)
        );

        assert(
            outputs.header.state.partial.public_data_tree.eq(inputs.previous_rollup_data[1].base_or_merge_rollup_public_inputs.end.public_data_tree)
        );
    }
}
