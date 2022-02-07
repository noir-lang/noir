#pragma once
#include "compute_circuit_data.hpp"
#include "verify.hpp"
#include "../inner_proof_data/inner_proof_data.hpp"
#include "../notes/native/defi_interaction/note.hpp"
#include "../../world_state/world_state.hpp"
#include <stdlib/merkle_tree/memory_store.hpp>

namespace rollup {
namespace proofs {
namespace root_rollup {

using WorldState = world_state::WorldState<plonk::stdlib::merkle_tree::MemoryStore>;

inline void pad_root_rollup_tx(root_rollup_tx& rollup, circuit_data const& circuit_data)
{
    rollup.rollups.resize(circuit_data.num_inner_rollups, circuit_data.inner_rollup_circuit_data.padding_proof);
    rollup.num_previous_defi_interactions = rollup.defi_interaction_notes.size();
    rollup.defi_interaction_notes.resize(NUM_BRIDGE_CALLS_PER_BLOCK);
    rollup.bridge_ids.resize(NUM_BRIDGE_CALLS_PER_BLOCK);
    rollup.asset_ids.resize(NUM_ASSETS, MAX_NUM_ASSETS);
}

inline root_rollup_tx create_root_rollup_tx(WorldState& world_state,
                                            uint32_t rollup_id,
                                            fr old_defi_root,
                                            std::vector<std::vector<uint8_t>> const& inner_rollups,
                                            std::vector<uint256_t> const& bridge_ids = {},
                                            std::vector<uint256_t> const& asset_ids = { 0 },
                                            std::vector<native::defi_interaction::note> const& interaction_notes = {},
                                            fr rollup_beneficiary = 0)
{
    auto& data_tree = world_state.data_tree;
    auto& root_tree = world_state.root_tree;
    auto& defi_tree = world_state.defi_tree;

    auto root_index = root_tree.size();

    root_rollup_tx tx;
    tx.rollup_id = rollup_id;
    tx.num_inner_proofs = static_cast<uint32_t>(inner_rollups.size());
    tx.rollups = inner_rollups;
    tx.old_data_roots_root = root_tree.root();
    tx.old_data_roots_path = root_tree.get_hash_path(root_index);
    auto data_root = data_tree.root();
    root_tree.update_element(root_index, data_root);
    tx.new_data_roots_root = root_tree.root();

    tx.old_defi_root = old_defi_root;
    tx.old_defi_path = defi_tree.get_hash_path(rollup_id ? rollup_id - 1 : 0);
    tx.new_defi_root = defi_tree.root();

    tx.bridge_ids = bridge_ids;
    tx.asset_ids = asset_ids;
    tx.defi_interaction_notes = interaction_notes;
    tx.rollup_beneficiary = rollup_beneficiary;
    return tx;
}

} // namespace root_rollup
} // namespace proofs
} // namespace rollup
