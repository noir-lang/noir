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
    rollup.new_null_roots.resize(rollup_size * 2, rollup.new_null_roots.back());
    rollup.old_null_paths.resize(rollup_size * 2, rollup.old_null_paths.back());
    rollup.data_roots_paths.resize(rollup_size, rollup.data_roots_paths.back());
    rollup.data_roots_indicies.resize(rollup_size, 0);
    rollup.num_defi_interactions = rollup.bridge_ids.size();
    rollup.bridge_ids.resize(NUM_BRIDGE_CALLS_PER_BLOCK);
    rollup.num_asset_ids = rollup.asset_ids.size();
    rollup.asset_ids.resize(NUM_ERC20_ASSETS);
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
                                  std::vector<uint256_t> asset_ids = {},
                                  std::vector<uint32_t> const& data_roots_indicies_ = {})
{
    auto& data_tree = world_state.data_tree;
    auto& null_tree = world_state.null_tree;
    auto& root_tree = world_state.root_tree;
    auto& defi_tree = world_state.defi_tree;

    uint32_t rollup_id = static_cast<uint32_t>(root_tree.size() - 1);

    auto floor_rollup_size = 1UL << numeric::get_msb(rollup_size);
    auto rollup_size_pow2 = floor_rollup_size << (rollup_size != floor_rollup_size);
    size_t rollup_tree_depth = numeric::get_msb(rollup_size_pow2) + 1;
    MemoryTree rollup_tree(rollup_tree_depth);

    // Compute data tree data.
    auto num_txs = static_cast<uint32_t>(txs.size());
    auto subtree_size = static_cast<uint32_t>(rollup_size_pow2 * 2UL);
    auto data_tree_size = static_cast<uint32_t>(data_tree.size());
    auto data_start_index = data_tree_size % subtree_size == 0
                                ? data_tree_size
                                : data_tree_size + subtree_size - (data_tree_size % subtree_size);
    auto old_data_root = data_tree.root();
    auto old_data_path = data_tree.get_hash_path(data_start_index);

    std::vector<fr_hash_path> data_roots_paths;
    std::vector<uint256_t> nullifier_indicies;
    std::vector<uint8_t> zero_value(64, 0);

    std::vector<uint32_t> data_roots_indicies(data_roots_indicies_);
    data_roots_indicies.resize(num_txs, (uint32_t)root_tree.size() - 1);

    for (size_t i = 0; i < num_txs; ++i) {
        auto proof_data = txs[i];
        auto struct_data = inner_proof_data(proof_data);

        if (struct_data.proof_id == ProofIds::DEFI_DEPOSIT) {
            uint32_t nonce = 0;
            while (struct_data.asset_id != bridge_ids[nonce] && nonce < bridge_ids.size()) {
                ++nonce;
            };
            nonce += rollup_id * NUM_BRIDGE_CALLS_PER_BLOCK;
            uint256_t fee = struct_data.tx_fee - (struct_data.tx_fee >> 1);
            struct_data.note_commitment1 =
                notes::native::claim::complete_partial_commitment(struct_data.note_commitment1, nonce, fee);
        }

        data_tree.update_element(data_start_index + i * 2, struct_data.note_commitment1);
        data_tree.update_element(data_start_index + i * 2 + 1, struct_data.note_commitment2);
        rollup_tree.update_element(i * 2, struct_data.note_commitment1);
        rollup_tree.update_element(i * 2 + 1, struct_data.note_commitment2);

        data_roots_paths.push_back(root_tree.get_hash_path(data_roots_indicies[i]));

        nullifier_indicies.push_back(uint256_t(struct_data.nullifier1));
        nullifier_indicies.push_back(uint256_t(struct_data.nullifier2));
    }

    // Compute nullifier tree data.
    auto old_null_root = null_tree.root();
    std::vector<fr> new_null_roots;
    std::vector<fr_hash_path> old_null_paths;
    std::vector<fr_hash_path> new_null_paths;

    auto nullifier_value = fr(1);

    for (size_t i = 0; i < nullifier_indicies.size(); ++i) {
        old_null_paths.push_back(null_tree.get_hash_path(nullifier_indicies[i]));
        if (nullifier_indicies[i]) {
            null_tree.update_element(nullifier_indicies[i], nullifier_value);
        }
        new_null_paths.push_back(null_tree.get_hash_path(nullifier_indicies[i]));
        new_null_roots.push_back(null_tree.root());
    }

    // Compute root tree data.
    auto root_tree_root = root_tree.root();

    // Compose our rollup.
    rollup_tx rollup = { rollup_id,           num_txs,          data_start_index, txs,
                         old_data_root,       data_tree.root(), old_data_path,    old_null_root,
                         new_null_roots,      old_null_paths,   root_tree_root,   data_roots_paths,
                         data_roots_indicies, defi_tree.root(), bridge_ids,       asset_ids,
                         bridge_ids.size(),   asset_ids.size() };

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
