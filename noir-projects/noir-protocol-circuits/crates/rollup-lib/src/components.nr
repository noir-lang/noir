use crate::abis::base_or_merge_rollup_public_inputs::BaseOrMergeRollupPublicInputs;
use crate::abis::previous_rollup_data::PreviousRollupData;
use dep::types::{
    hash::accumulate_sha256, merkle_tree::VariableMerkleTree,
    constants::{
    MAX_NOTE_HASHES_PER_TX, MAX_NULLIFIERS_PER_TX, MAX_L2_TO_L1_MSGS_PER_TX,
    MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PROTOCOL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX
},
    utils::{uint256::U256, arrays::array_length},
    abis::{
    append_only_tree_snapshot::AppendOnlyTreeSnapshot, accumulated_data::CombinedAccumulatedData,
    public_data_update_request::PublicDataUpdateRequest
}
};

/**
 * Asserts that the tree formed by rollup circuits is filled greedily from L to R
 *
 */
pub fn assert_txs_filled_from_left(
    left: BaseOrMergeRollupPublicInputs,
    right: BaseOrMergeRollupPublicInputs
) {
    // assert that the left rollup is either a base (1 tx) or a balanced tree (num txs = power of 2)
    if (left.rollup_type == 1) {
        let left_txs = left.num_txs;
        let right_txs = right.num_txs;
        // See https://graphics.stanford.edu/~seander/bithacks.html#DetermineIfPowerOf2
        assert(
            (left_txs) & (left_txs - 1) == 0, "The rollup should be filled greedily from L to R, but received an unbalanced left subtree"
        );
        assert(
            right_txs <= left_txs, "The rollup should be filled greedily from L to R, but received a L txs < R txs"
        );
    } else {
        assert(
            right.rollup_type == 0, "The rollup should be filled greedily from L to R, but received a L base and R merge"
        );
    }
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
        left.end.public_data_tree.eq(right.start.public_data_tree), "input proofs have different public data tree snapshots"
    );
}

pub fn accumulate_fees(
    left: BaseOrMergeRollupPublicInputs,
    right: BaseOrMergeRollupPublicInputs
) -> Field {
    left.accumulated_fees + right.accumulated_fees
}

/**
 * @brief From two previous rollup data, compute a single out hash
 *
 * @param previous_rollup_data
 * @return out hash stored in 2 fields
 */
pub fn compute_out_hash(previous_rollup_data: [PreviousRollupData; 2]) -> Field {
    accumulate_sha256(
        [
        previous_rollup_data[0].base_or_merge_rollup_public_inputs.out_hash,
        previous_rollup_data[1].base_or_merge_rollup_public_inputs.out_hash
    ]
    )
}

pub fn compute_kernel_out_hash(combined: CombinedAccumulatedData) -> Field {
    let non_empty_items = array_length(combined.l2_to_l1_msgs);
    let merkle_tree = VariableMerkleTree::new_sha(combined.l2_to_l1_msgs, non_empty_items);
    merkle_tree.get_root()
}

/**
 * @brief From two previous rollup data, compute a single txs effects hash
 *
 * @param previous_rollup_data
 * @return The hash of the transaction effects stored in 2 fields
 */
pub fn compute_txs_effects_hash(previous_rollup_data: [PreviousRollupData; 2]) -> Field {
    accumulate_sha256(
        [
        previous_rollup_data[0].base_or_merge_rollup_public_inputs.txs_effects_hash,
        previous_rollup_data[1].base_or_merge_rollup_public_inputs.txs_effects_hash
    ]
    )
}

// Tx effects hash consists of
// 1 field for revert code
// 1 field for transaction fee
// MAX_NOTE_HASHES_PER_TX fields for note hashes
// MAX_NULLIFIERS_PER_TX fields for nullifiers
// MAX_L2_TO_L1_MSGS_PER_TX fields for L2 to L1 messages
// MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX public data update requests -> MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX * 2 fields
//                                          __
// 1 note encrypted logs length --> 1 field   |
// 1 encrypted logs length --> 1 field        | -> 3 types of logs - 3 fields for its lengths
// 1 unencrypted logs length --> 1 field    __|
//                                                                                                                                   __
// 1 note encrypted logs hash --> 1 sha256 hash -> 31 bytes -> 1 fields | Beware when populating bytes that we fill (prepend) to 32!   |
// 1 encrypted logs hash --> 1 sha256 hash -> 31 bytes -> 1 fields | Beware when populating bytes that we fill (prepend) to 32!        | -> 3 types of logs - 3 fields for its hashes
// 1 unencrypted logs hash --> 1 sha256 hash -> 31 bytes  -> 1 fields | Beware when populating bytes that we fill (prepend) to 32!   __|
global TX_EFFECTS_HASH_INPUT_FIELDS = 1 + 1 + MAX_NOTE_HASHES_PER_TX + MAX_NULLIFIERS_PER_TX + MAX_L2_TO_L1_MSGS_PER_TX + MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX * 2 + 3 + 3;

// Computes the tx effects hash for a base rollup (a single transaction)
pub fn compute_tx_effects_hash(
    combined: CombinedAccumulatedData,
    revert_code: u8,
    transaction_fee: Field,
    all_public_data_update_requests: [PublicDataUpdateRequest; MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX]
) -> Field {
    let mut tx_effects_hash_input = [0; TX_EFFECTS_HASH_INPUT_FIELDS];

    let note_hashes = combined.note_hashes;
    let nullifiers = combined.nullifiers;
    let l2_to_l1_msgs = combined.l2_to_l1_msgs;

    // Public writes are the concatenation of all non-empty user update requests and protocol update requests, then padded with zeroes.
    // The incoming all_public_data_update_requests may have empty update requests in the middle, so we move those to the end of the array.
    let public_data_update_requests = get_all_update_requests_for_tx_effects(all_public_data_update_requests);
    let note_logs_length = combined.note_encrypted_log_preimages_length;
    let encrypted_logs_length = combined.encrypted_log_preimages_length;
    let unencrypted_logs_length = combined.unencrypted_log_preimages_length;
    let note_encrypted_logs_hash = combined.note_encrypted_logs_hash;
    let encrypted_logs_hash = combined.encrypted_logs_hash;
    let unencrypted_logs_hash = combined.unencrypted_logs_hash;

    let mut offset = 0;

    // REVERT CODE
    // upcast to Field to have the same size for the purposes of the hash
    tx_effects_hash_input[offset] = revert_code as Field;
    offset += 1;

    // TX FEE
    tx_effects_hash_input[offset] = transaction_fee;
    offset += 1;

    // NOTE HASHES
    for j in 0..MAX_NOTE_HASHES_PER_TX {
        tx_effects_hash_input[offset + j] = note_hashes[j];
    }
    offset += MAX_NOTE_HASHES_PER_TX ;

    // NULLIFIERS
    for j in 0..MAX_NULLIFIERS_PER_TX {
        tx_effects_hash_input[offset + j] = nullifiers[j];
    }
    offset += MAX_NULLIFIERS_PER_TX ;

    // L2 TO L1 MESSAGES
    for j in 0..MAX_L2_TO_L1_MSGS_PER_TX {
        tx_effects_hash_input[offset + j] = l2_to_l1_msgs[j];
    }
    offset += MAX_L2_TO_L1_MSGS_PER_TX;

    // PUBLIC DATA UPDATE REQUESTS
    for j in 0..MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX {
        tx_effects_hash_input[offset + j * 2] =
                public_data_update_requests[j].leaf_slot;
        tx_effects_hash_input[offset + j * 2 + 1] =
                public_data_update_requests[j].new_value;
    }
    offset += MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX * 2;

    // NOTE ENCRYPTED LOGS LENGTH
    tx_effects_hash_input[offset] = note_logs_length;
    offset += 1;

    // ENCRYPTED LOGS LENGTH
    tx_effects_hash_input[offset] = encrypted_logs_length;
    offset += 1;

    // UNENCRYPTED LOGS LENGTH
    tx_effects_hash_input[offset] = unencrypted_logs_length;
    offset += 1;

    // NOTE ENCRYPTED LOGS HASH
    tx_effects_hash_input[offset] = note_encrypted_logs_hash;
    offset += 1;

    // ENCRYPTED LOGS HASH
    tx_effects_hash_input[offset] = encrypted_logs_hash;
    offset += 1;

    // UNENCRYPTED LOGS HASH
    tx_effects_hash_input[offset] = unencrypted_logs_hash;
    offset += 1;

    assert_eq(offset, TX_EFFECTS_HASH_INPUT_FIELDS); // Sanity check

    let mut hash_input_flattened = [0; TX_EFFECTS_HASH_INPUT_FIELDS * 32];
    for offset in 0..TX_EFFECTS_HASH_INPUT_FIELDS {
        let input_as_bytes = tx_effects_hash_input[offset].to_be_bytes(32);
        for byte_index in 0..32 {
            hash_input_flattened[offset * 32 + byte_index] = input_as_bytes[byte_index];
        }
    }

    let sha_digest = dep::types::hash::sha256_to_field(hash_input_flattened);
    sha_digest
}

fn get_all_update_requests_for_tx_effects(all_public_data_update_requests: [PublicDataUpdateRequest; MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX]) -> [PublicDataUpdateRequest; MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX] {
    let mut all_update_requests: BoundedVec<PublicDataUpdateRequest, MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX> = BoundedVec::new();
    for update_request in all_public_data_update_requests {
        if !update_request.is_empty() {
            all_update_requests.push(update_request);
        }
    }
    all_update_requests.storage
}

#[test]
fn consistent_TX_EFFECTS_HASH_INPUT_FIELDS() {
    let expected_size = 1 // revert code
        + 1 // transaction fee
        + MAX_NOTE_HASHES_PER_TX
        + MAX_NULLIFIERS_PER_TX
        + MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX * 2
        + MAX_L2_TO_L1_MSGS_PER_TX
        + 3 // logs lengths
        + 3; // logs hashes
    assert(TX_EFFECTS_HASH_INPUT_FIELDS == expected_size, "tx effects hash input size is incorrect");
}
