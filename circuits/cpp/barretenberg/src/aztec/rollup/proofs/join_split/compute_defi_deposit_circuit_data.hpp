#pragma once
#include "compute_circuit_data.hpp"
#include "../../fixtures/user_context.hpp"
#include <stdlib/merkle_tree/merkle_tree.hpp>

namespace rollup {
namespace proofs {
namespace join_split {

using namespace plonk::stdlib::merkle_tree;

join_split_tx create_defi_deposit_tx(MerkleTree<MemoryStore>& data_tree,
                                     fixtures::user_context& user,
                                     uint32_t defi_deposit_amount,
                                     uint32_t change_amount);

} // namespace join_split
} // namespace proofs
} // namespace rollup