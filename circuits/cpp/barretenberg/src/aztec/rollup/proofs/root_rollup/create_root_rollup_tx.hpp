#pragma once
#include "compute_circuit_data.hpp"
#include "verify.hpp"
#include "../inner_proof_data.hpp"
#include <stdlib/merkle_tree/merkle_tree.hpp>
#include <stdlib/merkle_tree/memory_store.hpp>
#include <stdlib/merkle_tree/memory_tree.hpp>

namespace rollup {
namespace proofs {
namespace root_rollup {

using Tree = MerkleTree<MemoryStore>;

inline root_rollup_tx create_root_rollup_tx(uint32_t rollup_id,
                                            std::vector<std::vector<uint8_t>> const& inner_rollups,
                                            Tree& data_tree,
                                            Tree& root_tree)
{
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

    return tx_data;
}

} // namespace root_rollup
} // namespace proofs
} // namespace rollup
