#pragma once
#include "rollup_tx.hpp"
#include "../inner_proof_data.hpp"
#include "../../constants.hpp"
#include "../../world_state/world_state.hpp"
#include "../notes/native/claim/index.hpp"
#include <stdlib/merkle_tree/index.hpp>

namespace rollup {
namespace proofs {
namespace rollup {

using WorldState = world_state::WorldState<MemoryStore>;

/**
 * If `rollup` does not contain a full set of txs, we need to grow it with padding data.
 * This involves using the padding proof, which will always pass verification, but has garbage inputs.
 * The nullifier checks for padding proofs will actually check that index 0 "updates" from 0 to 0.
 * This requires the 0 index hash path for the padding proofs, passed as the last entry in the nullifier vectors.
 * This function grows the vectors to their full size.
 */
inline void pad_rollup_tx(rollup_tx& rollup, size_t rollup_size, std::vector<uint8_t> const& padding_proof)
{
    rollup.txs.resize(rollup_size, padding_proof);

    rollup.linked_commitment_paths.resize(
        rollup_size, fr_hash_path(DATA_TREE_DEPTH, std::make_pair(fr::random_element(), fr::random_element())));
    rollup.linked_commitment_indices.resize(rollup_size, 0);

    rollup.new_null_roots.resize(rollup_size * 2, rollup.new_null_roots.back());
    rollup.old_null_paths.resize(rollup_size * 2, rollup.old_null_paths.back());

    rollup.data_roots_paths.resize(rollup_size, rollup.data_roots_paths.back());
    rollup.data_roots_indicies.resize(rollup_size, 0);

    rollup.num_defi_interactions = rollup.bridge_ids.size();
    rollup.bridge_ids.resize(NUM_BRIDGE_CALLS_PER_BLOCK);
    rollup.num_asset_ids = rollup.asset_ids.size();
    rollup.asset_ids.resize(NUM_ASSETS);
}

/**
 * Creates a rollup_tx with the minimal amount of data to create valid empty rollup.
 * Must be padded with a call to `pad_rollup_tx()` before being given to the circuit.
 */
template <typename T> inline rollup_tx create_empty_rollup(T& world_state)
{
    // Compute data tree data.
    auto num_txs = 0U;
    auto data_start_index = 0U;
    auto data_root = world_state.data_tree.root();
    auto zero_data_path = world_state.data_tree.get_hash_path(0);
    auto null_root = world_state.null_tree.root();
    auto zero_null_path = world_state.null_tree.get_hash_path(0);
    auto roots_root = world_state.root_tree.root();
    auto zero_roots_path = world_state.root_tree.get_hash_path(0);

    // Compose our rollup.
    rollup_tx rollup = { .rollup_id = 0,
                         .num_txs = num_txs,
                         .data_start_index = data_start_index,
                         .txs = {},

                         .old_data_root = data_root,
                         .new_data_root = data_root,
                         .old_data_path = zero_data_path,

                         .linked_commitment_paths = { zero_data_path },
                         .linked_commitment_indices = { 0 },

                         .old_null_root = null_root,
                         .new_null_roots = { null_root },
                         .old_null_paths = { zero_null_path },

                         .data_roots_root = roots_root,
                         .data_roots_paths = { zero_roots_path },
                         .data_roots_indicies = { 0 },

                         .new_defi_root = world_state.defi_tree.root(),
                         .bridge_ids = {},
                         .asset_ids = {},
                         .num_defi_interactions = 0,
                         .num_asset_ids = 0 };

    return rollup;
}

/**
 * Create an empty, fully padded rollup_tx ready for use in the circuit.
 */
inline rollup_tx create_padding_rollup(size_t rollup_size, std::vector<uint8_t> const& padding_proof)
{
    world_state::WorldState<MemoryStore> world_state;
    auto rollup = create_empty_rollup(world_state);
    pad_rollup_tx(rollup, rollup_size, padding_proof);
    return rollup;
}

inline rollup_tx create_rollup_tx(WorldState& world_state,
                                  size_t rollup_size,
                                  std::vector<std::vector<uint8_t>> const& txs,
                                  std::vector<uint256_t> bridge_ids = {},
                                  std::vector<uint256_t> asset_ids = { 0 },
                                  std::vector<uint32_t> const& data_roots_indicies_ = {},
                                  std::vector<uint32_t> const& linked_commitment_indices_ = {})
{
    auto& data_tree = world_state.data_tree;
    auto& null_tree = world_state.null_tree;
    auto& root_tree = world_state.root_tree;
    auto& defi_tree = world_state.defi_tree;

    uint32_t rollup_id = static_cast<uint32_t>(root_tree.size() - 1);

    auto floor_rollup_size = 1UL << numeric::get_msb(rollup_size);
    auto rollup_size_pow2 = floor_rollup_size << (rollup_size != floor_rollup_size);

    // Compute data tree data.
    auto num_txs = static_cast<uint32_t>(txs.size());
    auto subtree_size = static_cast<uint32_t>(rollup_size_pow2 * 2UL);
    auto data_tree_size = static_cast<uint32_t>(data_tree.size());
    auto data_start_index = data_tree_size % subtree_size == 0
                                ? data_tree_size
                                : data_tree_size + subtree_size - (data_tree_size % subtree_size);
    auto old_data_root = data_tree.root();
    auto old_data_path = data_tree.get_hash_path(data_start_index);

    std::vector<fr_hash_path> linked_commitment_paths;
    std::vector<fr_hash_path> data_roots_paths;
    std::vector<uint256_t> nullifier_indicies;
    std::vector<fr> data_tree_values;

    std::vector<uint32_t> linked_commitment_indices(linked_commitment_indices_);
    linked_commitment_indices.resize(num_txs, data_tree_size - 1);
    std::vector<uint32_t> data_roots_indicies(data_roots_indicies_);
    data_roots_indicies.resize(num_txs, (uint32_t)root_tree.size() - 1);

    for (size_t i = 0; i < num_txs; ++i) {
        auto tx = inner_proof_data(txs[i]);

        // Chaining - the sole purpose of this 'if statement' is to 'zero' certain commitments and nullifiers in advance
        // of calculating the data_tree and null_tree roots.
        fr_hash_path linked_commitment_path;
        const bool chaining = tx.propagated_input_index > 0;
        bool is_propagating_prev_output1;
        bool is_propagating_prev_output2;
        if (chaining) {
            bool found_link_in_rollup = false;
            fr prev_allow_chain = 0;
            size_t matched_tx_index;
            // Loop through all prior txs to find a tx that this tx is chaining from (if it exists in this rollup):
            for (size_t j = 0; j < num_txs; j++) {
                const auto prev_tx = inner_proof_data(txs[j]);
                is_propagating_prev_output1 = prev_tx.note_commitment1 == tx.backward_link;
                is_propagating_prev_output2 = prev_tx.note_commitment2 == tx.backward_link;
                found_link_in_rollup = is_propagating_prev_output1 || is_propagating_prev_output2;
                if (found_link_in_rollup) {
                    prev_allow_chain = prev_tx.allow_chain;
                    matched_tx_index = j;
                    break;
                }
            }

            const bool start_of_subchain = !found_link_in_rollup;
            if (start_of_subchain) {
                // Then no earlier txs in this tx's chain have been included in this rollup, so we'll need to provide a
                // valid merkle membership witness for the input note being propagated:
                linked_commitment_paths.push_back(data_tree.get_hash_path(linked_commitment_indices[i]));
            } else {
                // This tx is not the first tx of its chain to be included in this rollup, hence the existence of the
                // input note being propagated is inductively assured by earlier checks in this circuit.
                if (i == 0) {
                    info(format(__FUNCTION__, "error, the 0th tx is never in the middle of a chain"));
                }

                linked_commitment_path = get_random_hash_path(data_tree.depth()); // create a dummy path.

                // Note: in the circuit, we do a check to ensure the commitment being propagaged (denoted by
                // `attempting_to_propagate_output_index`) is _allowed_ to be chained from, by comparing against
                // `prev_allow_chain`. We'll skip that check here, so that the circuit's checks can be tested.

                // Note: If we're in 'the middle' of a chain, and the user is chaining to themselves (always the case in
                // the current implementation), we can 'zero' the prev_tx's data tree values, and this tx's nullifiers.
                // Whilst we'll actually be passing the original nonzero values into the circuit, we need to calculate
                // the data tree root here as though they're zero.

                if (is_propagating_prev_output1) {
                    data_tree_values[2 * matched_tx_index] = fr(0);
                }
                if (is_propagating_prev_output2) {
                    data_tree_values[2 * matched_tx_index + 1] = fr(0);
                }
                if (tx.propagated_input_index == 1) {
                    tx.nullifier1 = 0;
                }
                if (tx.propagated_input_index == 2) {
                    tx.nullifier2 = 0;
                }
                // the data tree's root is calculated in line with these changes, later in this function.
            }
        } else {
            linked_commitment_path = get_random_hash_path(data_tree.depth()); // create an dummy path.
        }
        linked_commitment_paths.push_back(linked_commitment_path);

        // Compute partial claim notes
        if (tx.proof_id == ProofIds::DEFI_DEPOSIT) {
            uint32_t nonce = 0;
            while (tx.bridge_id != bridge_ids[nonce] && nonce < bridge_ids.size()) {
                ++nonce;
            };
            nonce += rollup_id * NUM_BRIDGE_CALLS_PER_BLOCK;
            uint256_t fee = tx.tx_fee - (tx.tx_fee >> 1);
            tx.note_commitment1 = notes::native::claim::complete_partial_commitment(tx.note_commitment1, nonce, fee);
        }

        data_tree_values.push_back(tx.note_commitment1);
        data_tree_values.push_back(tx.note_commitment2);

        data_roots_paths.push_back(root_tree.get_hash_path(data_roots_indicies[i]));

        nullifier_indicies.push_back(uint256_t(tx.nullifier1));
        nullifier_indicies.push_back(uint256_t(tx.nullifier2));
    }

    // Insert data tree elements.
    for (size_t i = 0; i < data_tree_values.size(); ++i) {
        if (data_tree_values[i] != fr(0)) {
            world_state.insert_data_entry(data_start_index + i, data_tree_values[i], nullifier_indicies[i]);
        }
    }

    // Compute nullifier tree data.
    auto old_null_root = null_tree.root();
    std::vector<fr> new_null_roots;
    std::vector<fr_hash_path> old_null_paths;

    auto nullifier_value = fr(1);

    for (size_t i = 0; i < nullifier_indicies.size(); ++i) {
        old_null_paths.push_back(null_tree.get_hash_path(nullifier_indicies[i]));
        if (nullifier_indicies[i]) {
            null_tree.update_element(nullifier_indicies[i], nullifier_value);
        }
        new_null_roots.push_back(null_tree.root());
    }

    // Compute root tree data.
    auto root_tree_root = root_tree.root();

    // Compose our rollup.
    rollup_tx rollup = { rollup_id,
                         num_txs,
                         data_start_index,
                         txs,

                         old_data_root,
                         data_tree.root(),
                         old_data_path,

                         linked_commitment_paths,
                         linked_commitment_indices,

                         old_null_root,
                         new_null_roots,
                         old_null_paths,

                         root_tree_root,
                         data_roots_paths,
                         data_roots_indicies,

                         defi_tree.root(),
                         bridge_ids,

                         asset_ids,

                         bridge_ids.size(),
                         asset_ids.size() };

    // Add nullifier 0 index padding data if necessary.
    if (num_txs < rollup_size) {
        data_tree.update_element(data_start_index + (rollup_size * 2) - 1, fr(0));

        auto zero_null_path = null_tree.get_hash_path(0);
        rollup.old_null_paths.push_back(zero_null_path);
    }

    return rollup;
}

} // namespace rollup
} // namespace proofs
} // namespace rollup
