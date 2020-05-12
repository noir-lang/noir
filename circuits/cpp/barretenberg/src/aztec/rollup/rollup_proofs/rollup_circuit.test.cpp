#include "compute_rollup_circuit_data.hpp"
#include "create_noop_join_split_proof.hpp"
#include "create_rollup.hpp"
#include "verify_rollup.hpp"
#include <stdlib/merkle_tree/leveldb_tree.hpp>
#include <stdlib/merkle_tree/memory_store.hpp>
#include <stdlib/merkle_tree/memory_tree.hpp>
#include <gtest/gtest.h>

using namespace barretenberg;
using namespace rollup::rollup_proofs;

TEST(rollup_proofs, test_1_proof_in_2_rollup)
{
    size_t num_txs = 1;
    size_t rollup_size = 2;
    MemoryStore store;
    MerkleTree data_tree(store, 32, 0);
    MerkleTree null_tree(store, 128, 1);

    auto rollup_circuit_data = compute_rollup_circuit_data(rollup_size);
    auto proof_data = create_noop_join_split_proof(data_tree.root()).proof_data;
    auto noop_proof_data = create_noop_join_split_proof(data_tree.root()).proof_data;
    auto txs = std::vector{ proof_data, noop_proof_data };

    auto rollup = create_rollup(num_txs, txs, data_tree, null_tree, rollup_size);

    auto verified = verify_rollup(rollup, rollup_circuit_data);

    EXPECT_TRUE(verified);
}