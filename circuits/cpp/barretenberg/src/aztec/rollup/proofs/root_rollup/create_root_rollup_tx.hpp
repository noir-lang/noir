#pragma once
#include "compute_circuit_data.hpp"
#include "verify.hpp"
#include "../inner_proof_data.hpp"
#include "../notes/native/defi_interaction/defi_interaction_note.hpp"
#include "../notes/native/defi_interaction/encrypt.hpp"
#include "../../world_state/world_state.hpp"
#include <stdlib/merkle_tree/memory_store.hpp>

namespace rollup {
namespace proofs {
namespace root_rollup {

using WorldState = world_state::WorldState<MemoryStore>;

inline void pad_rollup_tx(root_rollup_tx& rollup, circuit_data const& circuit_data)
{
    rollup.rollups.resize(circuit_data.num_inner_rollups, circuit_data.inner_rollup_circuit_data.padding_proof);
    rollup.num_defi_interactions = rollup.bridge_ids.size();
    rollup.bridge_ids.resize(NUM_BRIDGE_CALLS_PER_BLOCK);
    rollup.num_previous_defi_interactions = rollup.defi_interaction_notes.size();
    rollup.defi_interaction_notes.resize(NUM_BRIDGE_CALLS_PER_BLOCK);
}

inline root_rollup_tx create_root_rollup_tx(uint32_t rollup_id,
                                            std::vector<std::vector<uint8_t>> const& inner_rollups,
                                            WorldState& world_state)
{
    auto& data_tree = world_state.data_tree;
    auto& root_tree = world_state.root_tree;
    auto& defi_tree = world_state.defi_tree;

    auto root_index = root_tree.size();

    root_rollup_tx tx_data;
    tx_data.num_inner_proofs = static_cast<uint32_t>(inner_rollups.size());
    tx_data.rollup_id = rollup_id;
    tx_data.rollups = inner_rollups;
    tx_data.old_data_roots_root = root_tree.root();
    tx_data.old_data_roots_path = root_tree.get_hash_path(root_index);
    auto data_root = to_buffer(data_tree.root());
    root_tree.update_element(root_index, data_root);
    tx_data.new_data_roots_root = root_tree.root();
    tx_data.new_data_roots_path = root_tree.get_hash_path(root_index);

    tx_data.num_defi_interactions = 0;
    tx_data.old_defi_interaction_root = defi_tree.root();
    tx_data.old_defi_interaction_path = defi_tree.get_hash_path(defi_tree.size());
    tx_data.new_defi_interaction_root = defi_tree.root();
    tx_data.new_defi_interaction_path = defi_tree.get_hash_path(defi_tree.size());

    return tx_data;
}

} // namespace root_rollup
} // namespace proofs
} // namespace rollup
