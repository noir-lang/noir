#include "nullifier_memory_tree.hpp"
#include "../hash.hpp"

namespace bb::plonk {
namespace stdlib {
namespace merkle_tree {

NullifierMemoryTree::NullifierMemoryTree(size_t depth)
    : MemoryTree(depth)
{
    ASSERT(depth_ >= 1 && depth <= 32);
    total_size_ = 1UL << depth_;
    hashes_.resize(total_size_ * 2 - 2);

    // Build the entire tree and fill with 0 hashes.
    auto current = WrappedNullifierLeaf::zero().hash();
    size_t layer_size = total_size_;
    for (size_t offset = 0; offset < hashes_.size(); offset += layer_size, layer_size /= 2) {
        for (size_t i = 0; i < layer_size; ++i) {
            hashes_[offset + i] = current;
        }
        current = hash_pair_native(current, current);
    }

    // Insert the initial leaf at index 0
    auto initial_leaf = WrappedNullifierLeaf(nullifier_leaf{ .value = 0, .nextIndex = 0, .nextValue = 0 });
    leaves_.push_back(initial_leaf);
    root_ = update_element(0, initial_leaf.hash());
}

fr NullifierMemoryTree::update_element(fr const& value)
{
    // Find the leaf with the value closest and less than `value`

    // If value is 0 we simply append 0 a null NullifierLeaf to the tree
    if (value == 0) {
        auto zero_leaf = WrappedNullifierLeaf::zero();
        leaves_.push_back(zero_leaf);
        return update_element(leaves_.size() - 1, zero_leaf.hash());
    }

    size_t current;
    bool is_already_present;
    std::tie(current, is_already_present) = find_closest_leaf(leaves_, value);

    nullifier_leaf current_leaf = leaves_[current].unwrap();
    nullifier_leaf new_leaf = { .value = value,
                                .nextIndex = current_leaf.nextIndex,
                                .nextValue = current_leaf.nextValue };

    if (!is_already_present) {
        // Update the current leaf to point it to the new leaf
        current_leaf.nextIndex = leaves_.size();
        current_leaf.nextValue = value;

        leaves_[current].set(current_leaf);

        // Insert the new leaf with (nextIndex, nextValue) of the current leaf
        leaves_.push_back(new_leaf);
    }

    // Update the old leaf in the tree
    auto old_leaf_hash = current_leaf.hash();
    size_t old_leaf_index = current;
    auto root = update_element(old_leaf_index, old_leaf_hash);

    // Insert the new leaf in the tree
    auto new_leaf_hash = new_leaf.hash();
    size_t new_leaf_index = is_already_present ? old_leaf_index : leaves_.size() - 1;
    root = update_element(new_leaf_index, new_leaf_hash);

    return root;
}

} // namespace merkle_tree
} // namespace stdlib
} // namespace bb::plonk