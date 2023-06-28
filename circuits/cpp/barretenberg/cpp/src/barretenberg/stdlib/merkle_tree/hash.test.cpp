#include "hash.hpp"
#include "memory_tree.hpp"
#include <gtest/gtest.h>

#include "barretenberg/proof_system/circuit_builder/ultra_circuit_builder.hpp"
#include "barretenberg/stdlib/merkle_tree/membership.hpp"

namespace proof_system::stdlib_merkle_tree_hash_test {

using namespace barretenberg;
using namespace proof_system::plonk::stdlib;

using Composer = proof_system::UltraCircuitBuilder;

using field_ct = field_t<Composer>;
using witness_ct = witness_t<Composer>;

TEST(stdlib_merkle_tree_hash, compress_native_vs_circuit)
{
    fr x = uint256_t(0x5ec473eb273a8011, 0x50160109385471ca, 0x2f3095267e02607d, 0x02586f4a39e69b86);
    Composer composer = Composer();
    witness_ct y = witness_ct(&composer, x);
    field_ct z = pedersen_hash<Composer>::hash_multiple({ y, y });
    auto zz = merkle_tree::hash_pair_native(x, x);

    EXPECT_EQ(z.get_value(), zz);
}

TEST(stdlib_merkle_tree_hash, compute_tree_root_native_vs_circuit)
{
    Composer composer = Composer();
    std::vector<fr> inputs;
    std::vector<field_ct> inputs_ct;
    for (size_t i = 0; i < 16; i++) {
        auto input = fr::random_element();
        auto input_ct = witness_ct(&composer, input);
        inputs.push_back(input);
        inputs_ct.push_back(input_ct);
    }

    field_ct z = merkle_tree::compute_tree_root<Composer>(inputs_ct);
    auto zz = merkle_tree::compute_tree_root_native(inputs);

    EXPECT_EQ(z.get_value(), zz);
}

TEST(stdlib_merkle_tree_hash, compute_tree_native)
{
    constexpr size_t depth = 2;
    merkle_tree::MemoryTree mem_tree(depth);

    std::vector<fr> leaves;
    for (size_t i = 0; i < (size_t(1) << depth); i++) {
        auto input = fr::random_element();
        leaves.push_back(input);
        mem_tree.update_element(i, input);
    }

    std::vector<fr> tree_vector = merkle_tree::compute_tree_native(leaves);

    // Check if the tree vector matches the memory tree hashes
    for (size_t i = 0; i < tree_vector.size() - 1; i++) {
        EXPECT_EQ(tree_vector[i], mem_tree.hashes_[i]);
    }
    EXPECT_EQ(tree_vector.back(), mem_tree.root());
}
} // namespace proof_system::stdlib_merkle_tree_hash_test