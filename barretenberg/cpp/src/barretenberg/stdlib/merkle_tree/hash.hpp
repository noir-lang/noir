#pragma once
#include "barretenberg/common/net.hpp"
#include "barretenberg/crypto/blake2s/blake2s.hpp"
#include "barretenberg/crypto/pedersen_commitment/pedersen.hpp"
#include "barretenberg/crypto/pedersen_hash/pedersen.hpp"
#include "barretenberg/stdlib/hash/blake2s/blake2s.hpp"
#include "barretenberg/stdlib/hash/pedersen/pedersen.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"
#include <vector>

namespace bb::stdlib::merkle_tree {

inline bb::fr hash_pair_native(bb::fr const& lhs, bb::fr const& rhs)
{
    return crypto::pedersen_hash::hash({ lhs, rhs }); // uses lookup tables
}

inline bb::fr hash_native(std::vector<bb::fr> const& inputs)
{
    return crypto::pedersen_hash::hash(inputs); // uses lookup tables
}

/**
 * Computes the root of a tree with leaves given as the vector `input`.
 *
 * @param input: vector of leaf values.
 * @returns root as field
 */
inline bb::fr compute_tree_root_native(std::vector<bb::fr> const& input)
{
    // Check if the input vector size is a power of 2.
    ASSERT(input.size() > 0);
    ASSERT(numeric::is_power_of_two(input.size()));
    auto layer = input;
    while (layer.size() > 1) {
        std::vector<bb::fr> next_layer(layer.size() / 2);
        for (size_t i = 0; i < next_layer.size(); ++i) {
            next_layer[i] = crypto::pedersen_hash::hash({ layer[i * 2], layer[i * 2 + 1] });
        }
        layer = std::move(next_layer);
    }

    return layer[0];
}

// TODO write test
inline std::vector<bb::fr> compute_tree_native(std::vector<bb::fr> const& input)
{
    // Check if the input vector size is a power of 2.
    ASSERT(input.size() > 0);
    ASSERT(numeric::is_power_of_two(input.size()));
    auto layer = input;
    std::vector<bb::fr> tree(input);
    while (layer.size() > 1) {
        std::vector<bb::fr> next_layer(layer.size() / 2);
        for (size_t i = 0; i < next_layer.size(); ++i) {
            next_layer[i] = crypto::pedersen_hash::hash({ layer[i * 2], layer[i * 2 + 1] });
            tree.push_back(next_layer[i]);
        }
        layer = std::move(next_layer);
    }

    return tree;
}

} // namespace bb::stdlib::merkle_tree
