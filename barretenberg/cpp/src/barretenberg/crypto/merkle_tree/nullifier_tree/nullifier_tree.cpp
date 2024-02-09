#include "nullifier_tree.hpp"
#include "../hash.hpp"
#include "../memory_store.hpp"
#include "../merkle_tree.hpp"
#include "barretenberg/common/net.hpp"
#include "barretenberg/numeric/bitop/count_leading_zeros.hpp"
#include "barretenberg/numeric/bitop/keep_n_lsb.hpp"
#include "barretenberg/numeric/uint128/uint128.hpp"
#include <iostream>
#include <sstream>

namespace bb::crypto::merkle_tree {

template <typename Store, typename HashingPolicy>
NullifierTree<Store, HashingPolicy>::NullifierTree(Store& store, size_t depth, size_t initial_size, uint8_t tree_id)
    : MerkleTree<Store, HashingPolicy>(store, depth, tree_id)
{
    ASSERT(depth_ >= 1 && depth <= 256);
    ASSERT(initial_size > 0);
    zero_hashes_.resize(depth);

    // Create the zero hashes for the tree
    auto current =
        WrappedNullifierLeaf<HashingPolicy>(nullifier_leaf{ .value = 0, .nextIndex = 0, .nextValue = 0 }).hash();
    for (size_t i = 0; i < depth; ++i) {
        zero_hashes_[i] = current;
        current = HashingPolicy::hash_pair(current, current);
    }

    // Insert the initial leaves
    for (size_t i = 0; i < initial_size; i++) {
        auto initial_leaf =
            WrappedNullifierLeaf<HashingPolicy>(nullifier_leaf{ .value = i, .nextIndex = i + 1, .nextValue = i + 1 });
        leaves.push_back(initial_leaf);
    }

    leaves[initial_size - 1] = WrappedNullifierLeaf<HashingPolicy>(
        nullifier_leaf{ .value = leaves[initial_size - 1].unwrap().value, .nextIndex = 0, .nextValue = 0 });

    for (size_t i = 0; i < initial_size; ++i) {
        update_element(i, leaves[i].hash());
    }
}

template <typename Store, typename HashingPolicy>
NullifierTree<Store, HashingPolicy>::NullifierTree(NullifierTree&& other)
    : MerkleTree<Store, HashingPolicy>(std::move(other))
{}

template <typename Store, typename HashingPolicy> NullifierTree<Store, HashingPolicy>::~NullifierTree() {}

template <typename Store, typename HashingPolicy>
fr NullifierTree<Store, HashingPolicy>::update_element(fr const& value)
{
    // Find the leaf with the value closest and less than `value`
    size_t current;
    bool is_already_present;
    std::tie(current, is_already_present) = find_closest_leaf(leaves, value);

    nullifier_leaf current_leaf = leaves[current].unwrap();
    WrappedNullifierLeaf<HashingPolicy> new_leaf = WrappedNullifierLeaf<HashingPolicy>(
        { .value = value, .nextIndex = current_leaf.nextIndex, .nextValue = current_leaf.nextValue });
    if (!is_already_present) {
        // Update the current leaf to point it to the new leaf
        current_leaf.nextIndex = leaves.size();
        current_leaf.nextValue = value;

        leaves[current].set(current_leaf);

        // Insert the new leaf with (nextIndex, nextValue) of the current leaf
        leaves.push_back(new_leaf);
    }

    // Update the old leaf in the tree
    auto old_leaf_hash = leaves[current].hash();
    index_t old_leaf_index = current;
    auto r = update_element(old_leaf_index, old_leaf_hash);

    // Insert the new leaf in the tree
    auto new_leaf_hash = new_leaf.hash();
    index_t new_leaf_index = is_already_present ? old_leaf_index : leaves.size() - 1;
    r = update_element(new_leaf_index, new_leaf_hash);

    return r;
}

template class NullifierTree<MemoryStore, Poseidon2HashPolicy>;

} // namespace bb::crypto::merkle_tree