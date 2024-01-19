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

namespace bb::plonk {
namespace stdlib {
namespace merkle_tree {

using namespace bb;

template <typename T> inline bool bit_set(T const& index, size_t i)
{
    return bool((index >> i) & 0x1);
}

template <typename Store>
NullifierTree<Store>::NullifierTree(Store& store, size_t depth, uint8_t tree_id)
    : MerkleTree<Store>(store, depth, tree_id)
{
    ASSERT(depth_ >= 1 && depth <= 256);
    zero_hashes_.resize(depth);

    // Compute the zero values at each layer.
    // Insert the zero leaf to the `leaves` and also to the tree at index 0.
    WrappedNullifierLeaf initial_leaf =
        WrappedNullifierLeaf(nullifier_leaf{ .value = 0, .nextIndex = 0, .nextValue = 0 });
    leaves.push_back(initial_leaf);
    update_element(0, initial_leaf.hash());

    // Create the zero hashes for the tree
    auto current = WrappedNullifierLeaf::zero().hash();
    for (size_t i = 0; i < depth; ++i) {
        zero_hashes_[i] = current;
        current = hash_pair_native(current, current);
    }
}

template <typename Store>
NullifierTree<Store>::NullifierTree(NullifierTree&& other)
    : MerkleTree<Store>(std::move(other))
{}

template <typename Store> NullifierTree<Store>::~NullifierTree() {}

template <typename Store> fr NullifierTree<Store>::update_element(fr const& value)
{
    // Find the leaf with the value closest and less than `value`
    size_t current;
    bool is_already_present;
    std::tie(current, is_already_present) = find_closest_leaf(leaves, value);

    nullifier_leaf current_leaf = leaves[current].unwrap();
    WrappedNullifierLeaf new_leaf = WrappedNullifierLeaf(
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

template class NullifierTree<MemoryStore>;

} // namespace merkle_tree
} // namespace stdlib
} // namespace bb::plonk