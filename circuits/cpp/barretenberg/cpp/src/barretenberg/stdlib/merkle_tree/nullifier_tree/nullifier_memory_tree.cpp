#include "nullifier_memory_tree.hpp"
#include "../hash.hpp"

namespace proof_system::plonk {
namespace stdlib {
namespace merkle_tree {

NullifierMemoryTree::NullifierMemoryTree(size_t depth)
    : MemoryTree(depth)
{
    ASSERT(depth_ >= 1 && depth <= 32);
    total_size_ = 1UL << depth_;
    hashes_.resize(total_size_ * 2 - 2);

    // Build the entire tree.
    nullifier_leaf zero_leaf = { 0, 0, 0 };
    leaves_.push_back(zero_leaf);
    auto current = zero_leaf.hash();
    update_element(0, current);
    size_t layer_size = total_size_;
    for (size_t offset = 0; offset < hashes_.size(); offset += layer_size, layer_size /= 2) {
        for (size_t i = 0; i < layer_size; ++i) {
            hashes_[offset + i] = current;
        }
        current = hash_pair_native(current, current);
    }

    root_ = current;
}

fr NullifierMemoryTree::update_element(fr const& value)
{
    // Find the leaf with the value closest and less than `value`
    size_t current;
    bool is_already_present;
    std::tie(current, is_already_present) = find_closest_leaf(leaves_, value);

    nullifier_leaf new_leaf = { .value = value,
                                .nextIndex = leaves_[current].nextIndex,
                                .nextValue = leaves_[current].nextValue };
    if (!is_already_present) {
        // Update the current leaf to point it to the new leaf
        leaves_[current].nextIndex = leaves_.size();
        leaves_[current].nextValue = value;

        // Insert the new leaf with (nextIndex, nextValue) of the current leaf
        leaves_.push_back(new_leaf);
    }

    // Update the old leaf in the tree
    auto old_leaf_hash = leaves_[current].hash();
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
} // namespace proof_system::plonk