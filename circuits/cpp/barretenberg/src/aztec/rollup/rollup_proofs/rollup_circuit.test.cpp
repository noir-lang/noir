#include "compute_rollup_circuit_data.hpp"
#include "create_noop_join_split_proof.hpp"
#include "verify_rollup.hpp"
#include <stdlib/merkle_tree/leveldb_tree.hpp>
#include <stdlib/merkle_tree/memory_store.hpp>
#include <stdlib/merkle_tree/memory_tree.hpp>
#include <gtest/gtest.h>

using namespace barretenberg;
using namespace rollup::rollup_proofs;

TEST(rollup_proofs, test_rollup_1_proofs)
{
    size_t rollup_size = 1;
    size_t rollup_tree_depth = numeric::get_msb(rollup_size) + 1;
    MemoryStore store;

    // Compute data tree data.
    MerkleTree data_tree(store, 32, 0);
    MemoryTree rollup_tree(rollup_tree_depth);

    auto old_data_root = data_tree.root();
    auto old_data_path = data_tree.get_hash_path(0);

    auto proof_data = create_noop_join_split_proof(old_data_root).proof_data;
    auto rollup_circuit_data = compute_rollup_circuit_data(rollup_size);

    auto data_value1 =
        std::vector(proof_data.begin() + 2 * 32, proof_data.begin() + 2 * 32 + 64);
    auto data_value2 =
        std::vector(proof_data.begin() + 4 * 32, proof_data.begin() + 4 * 32 + 64);

    data_tree.update_element(0, data_value1);
    data_tree.update_element(1, data_value2);
    rollup_tree.update_element(0, data_value1);
    rollup_tree.update_element(1, data_value2);

    // Compute nullifier tree data.
    std::vector<uint128_t> nullifier_indicies = {
        from_buffer<uint128_t>(std::vector(proof_data.begin() + 7 * 32 + 16, proof_data.begin() + 7 * 32 + 32)),
        from_buffer<uint128_t>(std::vector(proof_data.begin() + 8 * 32 + 16, proof_data.begin() + 8 * 32 + 32)),
    };
    MerkleTree null_tree(store, 128, 1);

    auto old_null_root = null_tree.root();
    std::vector<fr> new_null_roots;
    std::vector<fr_hash_path> old_null_paths;
    std::vector<fr_hash_path> new_null_paths;

    auto nullifier_value = std::vector<uint8_t>(64, 0);
    nullifier_value[63] = 1;

    for (size_t i=0; i < nullifier_indicies.size(); ++i) {
        old_null_paths.push_back(null_tree.get_hash_path(nullifier_indicies[i]));
        null_tree.update_element(nullifier_indicies[i], nullifier_value);
        new_null_paths.push_back(null_tree.get_hash_path(nullifier_indicies[i]));
        new_null_roots.push_back(null_tree.root());
    }

    // Compose our rollup.
    rollup_tx rollup = {
        0,
        (uint32_t)rollup_size,
        (uint32_t)proof_data.size(),
        0,
        std::vector(rollup_size, proof_data),
        rollup_tree.root(),
        old_data_root,
        data_tree.root(),
        old_data_path,
        data_tree.get_hash_path(0),
        old_null_root,
        new_null_roots,
        old_null_paths,
        new_null_paths,
    };

    auto verified = verify_rollup(rollup, rollup_circuit_data);

    EXPECT_TRUE(verified);
}