use crate::abis::base_or_merge_rollup_public_inputs::BaseOrMergeRollupPublicInputs;
use crate::abis::previous_rollup_data::PreviousRollupData;
use dep::types::{
    mocked::AggregationObject, hash::accumulate_sha256,
    constants::{
    NUM_FIELDS_PER_SHA256, MAX_NEW_NOTE_HASHES_PER_TX, MAX_NEW_NULLIFIERS_PER_TX,
    MAX_NEW_L2_TO_L1_MSGS_PER_TX, NUM_UNENCRYPTED_LOGS_HASHES_PER_TX, NUM_ENCRYPTED_LOGS_HASHES_PER_TX,
    MAX_NEW_CONTRACTS_PER_TX, MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX
},
    utils::uint256::U256,
    abis::{append_only_tree_snapshot::AppendOnlyTreeSnapshot, accumulated_data::CombinedAccumulatedData}
};

/**
 * Create an aggregation object for the proofs that are provided
 *          - We add points P0 for each of our proofs
 *          - We add points P1 for each of our proofs
 *          - We concat our public inputs
 * TODO(Kev): This seems similar to the aggregate_proof method in the private-kernel-lib
 */
pub fn aggregate_proofs(
    left: BaseOrMergeRollupPublicInputs,
    _right: BaseOrMergeRollupPublicInputs
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
        left.height_in_block_tree == right.height_in_block_tree, "input proofs are of different rollup heights"
    );
    left.height_in_block_tree
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
 * @brief From two previous rollup data, compute a single out hash
 *
 * @param previous_rollup_data
 * @return out hash stored in 2 fields
 */
pub fn compute_out_hash(previous_rollup_data: [PreviousRollupData; 2]) -> [Field; NUM_FIELDS_PER_SHA256] {
    accumulate_sha256(
        [
        U128::from_integer(previous_rollup_data[0].base_or_merge_rollup_public_inputs.out_hash[0]),
        U128::from_integer(previous_rollup_data[0].base_or_merge_rollup_public_inputs.out_hash[1]),
        U128::from_integer(previous_rollup_data[1].base_or_merge_rollup_public_inputs.out_hash[0]),
        U128::from_integer(previous_rollup_data[1].base_or_merge_rollup_public_inputs.out_hash[1])
    ]
    )
}

pub fn compute_kernel_out_hash(combined: CombinedAccumulatedData) -> [Field; NUM_FIELDS_PER_SHA256] {
    let mut out_hash_inputs: [Field; MAX_NEW_L2_TO_L1_MSGS_PER_TX] = combined.new_l2_to_l1_msgs;

    let mut hash_input_flattened = [0; MAX_NEW_L2_TO_L1_MSGS_PER_TX * 32];
    for offset in 0..MAX_NEW_L2_TO_L1_MSGS_PER_TX {
        let input_as_bytes = out_hash_inputs[offset].to_be_bytes(32);
        for byte_index in 0..32 {
            hash_input_flattened[offset * 32 + byte_index] = input_as_bytes[byte_index];
        }
    }

    let sha_digest = dep::std::hash::sha256(hash_input_flattened);
    U256::from_bytes32(sha_digest).to_u128_limbs()
}

/**
 * @brief From two previous rollup data, compute a single txs effects hash
 *
 * @param previous_rollup_data
 * @return The hash of the transaction effects stored in 2 fields
 */
pub fn compute_txs_effects_hash(previous_rollup_data: [PreviousRollupData; 2]) -> [Field; NUM_FIELDS_PER_SHA256] {
    accumulate_sha256(
        [
        U128::from_integer(previous_rollup_data[0].base_or_merge_rollup_public_inputs.txs_effects_hash[0]),
        U128::from_integer(previous_rollup_data[0].base_or_merge_rollup_public_inputs.txs_effects_hash[1]),
        U128::from_integer(previous_rollup_data[1].base_or_merge_rollup_public_inputs.txs_effects_hash[0]),
        U128::from_integer(previous_rollup_data[1].base_or_merge_rollup_public_inputs.txs_effects_hash[1])
    ]
    )
}

global TX_EFFECTS_HASH_FULL_FIELDS = 197;
global TX_EFFECTS_HASH_LOG_FIELDS = 4;
global TX_EFFECTS_HASH_INPUT_FIELDS = 201; // TX_EFFECTS_HASH_FULL_FIELDS + TX_EFFECTS_HASH_LOG_FIELDS

// Computes the tx effects hash for a base rollup (a single transaction)
// TODO(Alvaro): This is too slow for brillig without the array optimization
pub fn compute_tx_effects_hash(combined: CombinedAccumulatedData) -> [Field; NUM_FIELDS_PER_SHA256] {
    // Compute tx effect hash
    // Consist of
    // MAX_NEW_NOTE_HASHES_PER_TX fields for note hashes
    // MAX_NEW_NULLIFIERS_PER_TX fields for nullifiers
    // 2 l2 -> l1 messages -> 2 fields
    // 32 public data update requests -> 64 fields
    // 1 contract deployments -> 3 fields
    // 1 encrypted logs hash --> 1 sha256 hash -> 32 bytes -> 2 fields | Beware when populating bytes that it is only 32!  
    // 1 unencrypted logs hash --> 1 sha256 hash -> 32 bytes  -> 2 fields | Beware when populating bytes that it is only 32!
    let mut txs_effects_hash_input = [0; TX_EFFECTS_HASH_INPUT_FIELDS];

    let new_note_hashes = combined.new_note_hashes;
    let new_nullifiers = combined.new_nullifiers;
    let newL2ToL1msgs = combined.new_l2_to_l1_msgs;
    let public_data_update_requests = combined.public_data_update_requests;
    let encryptedLogsHash = combined.encrypted_logs_hash;
    let unencryptedLogsHash = combined.unencrypted_logs_hash;

    let mut offset = 0;

    for j in 0..MAX_NEW_NOTE_HASHES_PER_TX {
        txs_effects_hash_input[offset + j] = new_note_hashes[j].value;
    }
    offset += MAX_NEW_NOTE_HASHES_PER_TX ;

    for j in 0..MAX_NEW_NULLIFIERS_PER_TX {
        txs_effects_hash_input[offset + j] = new_nullifiers[j].value;
    }
    offset += MAX_NEW_NULLIFIERS_PER_TX ;

    for j in 0..MAX_NEW_L2_TO_L1_MSGS_PER_TX {
        txs_effects_hash_input[offset + j] = newL2ToL1msgs[j];
    }
    offset += MAX_NEW_L2_TO_L1_MSGS_PER_TX;

    for j in 0..MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX {
        txs_effects_hash_input[offset + j * 2] =
                public_data_update_requests[j].leaf_slot;
        txs_effects_hash_input[offset + j * 2 + 1] =
                public_data_update_requests[j].new_value;
    }
    offset += MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX * 2;

    let contract_leaf = combined.new_contracts[0];
    txs_effects_hash_input[offset] = contract_leaf.hash();

    offset += MAX_NEW_CONTRACTS_PER_TX;

    let new_contracts = combined.new_contracts;
    txs_effects_hash_input[offset] = new_contracts[0].contract_address.to_field();
    txs_effects_hash_input[offset + 1] = new_contracts[0].portal_contract_address.to_field();

    offset += MAX_NEW_CONTRACTS_PER_TX * 2;

    for j in 0..NUM_FIELDS_PER_SHA256 {
        txs_effects_hash_input[offset + j] = encryptedLogsHash[j];
    }

    offset += NUM_ENCRYPTED_LOGS_HASHES_PER_TX * NUM_FIELDS_PER_SHA256;

    for j in 0..NUM_FIELDS_PER_SHA256 {
        txs_effects_hash_input[offset + j] = unencryptedLogsHash[j];
    }

    offset += NUM_UNENCRYPTED_LOGS_HASHES_PER_TX * NUM_FIELDS_PER_SHA256;
    assert_eq(offset, TX_EFFECTS_HASH_INPUT_FIELDS); // Sanity check

    let mut hash_input_flattened = [0; TX_EFFECTS_HASH_FULL_FIELDS * 32 + TX_EFFECTS_HASH_LOG_FIELDS * 16];
    for offset in 0..TX_EFFECTS_HASH_FULL_FIELDS {
        let input_as_bytes = txs_effects_hash_input[offset].to_be_bytes(32);
        for byte_index in 0..32 {
            hash_input_flattened[offset * 32 + byte_index] = input_as_bytes[byte_index];
        }
    }

    for log_field_index in 0..TX_EFFECTS_HASH_LOG_FIELDS {
        let input_as_bytes = txs_effects_hash_input[TX_EFFECTS_HASH_FULL_FIELDS + log_field_index].to_be_bytes(16);
        for byte_index in 0..16 {
            hash_input_flattened[TX_EFFECTS_HASH_FULL_FIELDS * 32 + log_field_index * 16 + byte_index] = input_as_bytes[byte_index];
        }
    }

    let sha_digest = dep::std::hash::sha256(hash_input_flattened);
    U256::from_bytes32(sha_digest).to_u128_limbs()
}

#[test]
fn consistent_TX_EFFECTS_HASH_INPUT_FIELDS() {
    let expected_size = MAX_NEW_NOTE_HASHES_PER_TX
        + MAX_NEW_NULLIFIERS_PER_TX
        + MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX * 2
        + MAX_NEW_L2_TO_L1_MSGS_PER_TX
        + MAX_NEW_CONTRACTS_PER_TX * 3
        + NUM_ENCRYPTED_LOGS_HASHES_PER_TX * NUM_FIELDS_PER_SHA256
        + NUM_UNENCRYPTED_LOGS_HASHES_PER_TX * NUM_FIELDS_PER_SHA256;
    assert(TX_EFFECTS_HASH_INPUT_FIELDS == expected_size, "tx effects hash input size is incorrect");
}

#[test]
fn consistent_tx_effects_hash_log_input_size() {
    assert_eq(
        TX_EFFECTS_HASH_LOG_FIELDS, NUM_ENCRYPTED_LOGS_HASHES_PER_TX * NUM_FIELDS_PER_SHA256
        + NUM_UNENCRYPTED_LOGS_HASHES_PER_TX * NUM_FIELDS_PER_SHA256, "tx effects hash log input field size is incorrect"
    );
}

#[test]
fn consistent_tx_effects_input_size() {
    assert_eq(
        TX_EFFECTS_HASH_INPUT_FIELDS, TX_EFFECTS_HASH_FULL_FIELDS + TX_EFFECTS_HASH_LOG_FIELDS, "tx effects hash input field size is incorrect"
    );
}

