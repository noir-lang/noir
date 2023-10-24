#pragma once
#include "barretenberg/common/net.hpp"
#include "barretenberg/crypto/blake2s/blake2s.hpp"
#include "barretenberg/crypto/pedersen_commitment/pedersen.hpp"
#include "barretenberg/crypto/pedersen_hash/pedersen.hpp"
#include "barretenberg/stdlib/hash/blake2s/blake2s.hpp"
#include "barretenberg/stdlib/hash/pedersen/pedersen.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"
#include <vector>

namespace proof_system::plonk::stdlib::merkle_tree {

inline barretenberg::fr hash_pair_native(barretenberg::fr const& lhs, barretenberg::fr const& rhs)
{
    return crypto::pedersen_hash::hash({ lhs, rhs }); // uses lookup tables
}

inline barretenberg::fr hash_native(std::vector<barretenberg::fr> const& inputs)
{
    return crypto::pedersen_hash::hash(inputs); // uses lookup tables
}

/**
 * Computes the root of a tree with leaves given as the vector `input`.
 *
 * @param input: vector of leaf values.
 * @returns root as field
 */
inline barretenberg::fr compute_tree_root_native(std::vector<barretenberg::fr> const& input)
{
    // Check if the input vector size is a power of 2.
    ASSERT(input.size() > 0);
    ASSERT(numeric::is_power_of_two(input.size()));
    auto layer = input;
    while (layer.size() > 1) {
        std::vector<barretenberg::fr> next_layer(layer.size() / 2);
        for (size_t i = 0; i < next_layer.size(); ++i) {
            next_layer[i] = crypto::pedersen_hash::hash({ layer[i * 2], layer[i * 2 + 1] });
        }
        layer = std::move(next_layer);
    }

    return layer[0];
}

// TODO write test
inline std::vector<barretenberg::fr> compute_tree_native(std::vector<barretenberg::fr> const& input)
{
    // Check if the input vector size is a power of 2.
    ASSERT(input.size() > 0);
    ASSERT(numeric::is_power_of_two(input.size()));
    auto layer = input;
    std::vector<barretenberg::fr> tree(input);
    while (layer.size() > 1) {
        std::vector<barretenberg::fr> next_layer(layer.size() / 2);
        for (size_t i = 0; i < next_layer.size(); ++i) {
            next_layer[i] = crypto::pedersen_hash::hash({ layer[i * 2], layer[i * 2 + 1] });
            tree.push_back(next_layer[i]);
        }
        layer = std::move(next_layer);
    }

    return tree;
}

} // namespace proof_system::plonk::stdlib::merkle_tree
