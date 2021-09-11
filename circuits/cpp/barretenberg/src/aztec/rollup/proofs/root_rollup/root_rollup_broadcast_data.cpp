#include "root_rollup_broadcast_data.hpp"
#include "../inner_proof_data.hpp"
#include "../../constants.hpp"
#include <crypto/sha256/sha256.hpp>
#include <common/container.hpp>

namespace rollup {
namespace proofs {
namespace root_rollup {

root_rollup_broadcast_data::root_rollup_broadcast_data(std::vector<fr> const& fields)
{
    rollup_id = fields[RootRollupBroadcastFields::ROLLUP_ID];
    rollup_size = fields[RootRollupBroadcastFields::ROLLUP_SIZE];
    data_start_index = fields[RootRollupBroadcastFields::DATA_START_INDEX];
    old_data_root = fields[RootRollupBroadcastFields::OLD_DATA_ROOT];
    new_data_root = fields[RootRollupBroadcastFields::NEW_DATA_ROOT];
    old_null_root = fields[RootRollupBroadcastFields::OLD_NULL_ROOT];
    new_null_root = fields[RootRollupBroadcastFields::NEW_NULL_ROOT];
    old_data_roots_root = fields[RootRollupBroadcastFields::OLD_DATA_ROOTS_ROOT];
    new_data_roots_root = fields[RootRollupBroadcastFields::NEW_DATA_ROOTS_ROOT];
    old_defi_root = fields[RootRollupBroadcastFields::OLD_DEFI_ROOT];
    new_defi_root = fields[RootRollupBroadcastFields::NEW_DEFI_ROOT];
    for (size_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; ++i) {
        bridge_ids[i] = fields[RootRollupBroadcastFields::DEFI_BRIDGE_IDS + i];
    }
    for (size_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; ++i) {
        deposit_sums[i] = fields[RootRollupBroadcastFields::DEFI_BRIDGE_DEPOSITS + i];
    }
    for (size_t i = 0; i < NUM_ASSETS; ++i) {
        asset_ids[i] = fields[RootRollupBroadcastFields::ASSET_IDS + i];
    }
    for (size_t i = 0; i < NUM_ASSETS; ++i) {
        total_tx_fees[i] = fields[RootRollupBroadcastFields::TOTAL_TX_FEES + i];
    }
    for (size_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; ++i) {
        defi_interaction_notes[i] = fields[RootRollupBroadcastFields::DEFI_INTERACTION_NOTES + i];
    }
    previous_defi_interaction_hash = fields[RootRollupBroadcastFields::PREVIOUS_DEFI_INTERACTION_HASH];
    num_inner_proofs = static_cast<uint32_t>(fields[RootRollupBroadcastFields::NUM_INNER_PROOFS]);

    size_t size = static_cast<uint32_t>(rollup_size);
    tx_data.resize(size);
    for (size_t i = 0; i < size; ++i) {
        auto offset =
            RootRollupBroadcastFields::INNER_PROOFS_DATA + (i * rollup::PropagatedInnerProofFields::NUM_FIELDS);
        tx_data[i].proof_id = fields[offset + InnerProofFields::PROOF_ID];
        tx_data[i].public_input = fields[offset + InnerProofFields::PUBLIC_INPUT];
        tx_data[i].public_output = fields[offset + InnerProofFields::PUBLIC_OUTPUT];
        tx_data[i].asset_id = fields[offset + InnerProofFields::ASSET_ID];
        tx_data[i].note_commitment1 = fields[offset + InnerProofFields::NOTE_COMMITMENT1];
        tx_data[i].note_commitment2 = fields[offset + InnerProofFields::NOTE_COMMITMENT2];
        tx_data[i].nullifier1 = fields[offset + InnerProofFields::NULLIFIER1];
        tx_data[i].nullifier2 = fields[offset + InnerProofFields::NULLIFIER2];
        tx_data[i].input_owner = fields[offset + InnerProofFields::INPUT_OWNER];
        tx_data[i].output_owner = fields[offset + InnerProofFields::OUTPUT_OWNER];
    }
}

fr root_rollup_broadcast_data::compute_hash() const
{
    // Slice off the fields representing the tx public inputs.
    std::vector<uint8_t> hash_inputs = slice(to_buffer(*this), 0, RootRollupBroadcastFields::INNER_PROOFS_DATA * 32);

    // Write the hashes representing the tx public inputs.
    size_t num_inner_rollups = static_cast<uint32_t>(num_inner_proofs);
    size_t num_txs_per_rollup = static_cast<uint32_t>(rollup_size) / num_inner_rollups;
    for (size_t i = 0; i < num_inner_rollups; ++i) {
        std::vector<uint8_t> inner_inputs;
        for (size_t j = 0; j < num_txs_per_rollup; ++j) {
            write(inner_inputs, tx_data[i * num_txs_per_rollup + j]);
        }
        auto inner_hash = sha256::sha256_to_field(inner_inputs);
        write(hash_inputs, inner_hash);
    }

    return sha256::sha256_to_field(hash_inputs);
}

} // namespace root_rollup
} // namespace proofs
} // namespace rollup
