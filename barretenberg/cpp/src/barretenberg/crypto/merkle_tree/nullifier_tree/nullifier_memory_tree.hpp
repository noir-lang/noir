#pragma once
#include "../hash.hpp"
#include "../memory_tree.hpp"
#include "nullifier_leaf.hpp"

namespace bb::crypto::merkle_tree {

/**
 * An NullifierMemoryTree is structured just like a usual merkle tree:
 *
 *                                       hashes_
 *    +------------------------------------------------------------------------------+
 *    |  0 -> h_{0,0}  h_{0,1}  h_{0,2}  h_{0,3}  h_{0,4}  h_{0,5}  h_{0,6}  h_{0,7} |
 *  i |                                                                              |
 *  n |  8 -> h_{1,0}  h_{1,1}  h_{1,2}  h_{1,3}                                     |
 *  d |                                                                              |
 *  e | 12 -> h_{2,0}  h_{2,1}                                                       |
 *  x |                                                                              |
 *    | 14 -> h_{3,0}                                                                |
 *    +------------------------------------------------------------------------------+
 *
 * Here, depth_ = 3 and {h_{0,j}}_{i=0..7} are leaf values.
 * Also, root_ = h_{3,0} and total_size_ = (2 * 8 - 2) = 14.
 * Lastly, h_{i,j} = hash( h_{i-1,2j}, h_{i-1,2j+1} ) where i > 1.
 *
 * 1. Initial state:
 *
 *                                        #
 *
 *                        #                               #
 *
 *                #               #               #               #
 *
 *            #       #       #       #        #       #       #       #
 *
 *  index     0       1       2       3        4       5       6       7
 *
 *  val       0       0       0       0        0       0       0       0
 *  nextIdx   0       0       0       0        0       0       0       0
 *  nextVal   0       0       0       0        0       0       0       0
 *
 * 2. Add new leaf with value 30
 *
 *  val       0       30      0       0        0       0       0       0
 *  nextIdx   1       0       0       0        0       0       0       0
 *  nextVal   30      0       0       0        0       0       0       0
 *
 * 3. Add new leaf with value 10
 *
 *  val       0       30      10      0        0       0       0       0
 *  nextIdx   2       0       1       0        0       0       0       0
 *  nextVal   10      0       30      0        0       0       0       0
 *
 * 4. Add new leaf with value 20
 *
 *  val       0       30      10      20       0       0       0       0
 *  nextIdx   2       0       3       1        0       0       0       0
 *  nextVal   10      0       20      30       0       0       0       0
 *
 * 5. Add new leaf with value 50
 *
 *  val       0       30      10      20       50      0       0       0
 *  nextIdx   2       4       3       1        0       0       0       0
 *  nextVal   10      50      20      30       0       0       0       0
 */
template <typename HashingPolicy> class NullifierMemoryTree : public MemoryTree<HashingPolicy> {

  public:
    NullifierMemoryTree(size_t depth, size_t initial_size = 1);

    using MemoryTree<HashingPolicy>::get_hash_path;
    using MemoryTree<HashingPolicy>::root;
    using MemoryTree<HashingPolicy>::update_element;

    fr_hash_path update_element(fr const& value);

    const std::vector<bb::fr>& get_hashes() { return hashes_; }
    const WrappedNullifierLeaf<HashingPolicy> get_leaf(size_t index)
    {
        return (index < leaves_.size()) ? leaves_[index] : WrappedNullifierLeaf<HashingPolicy>(nullifier_leaf::zero());
    }
    const std::vector<WrappedNullifierLeaf<HashingPolicy>>& get_leaves() { return leaves_; }

  protected:
    using MemoryTree<HashingPolicy>::depth_;
    using MemoryTree<HashingPolicy>::hashes_;
    using MemoryTree<HashingPolicy>::root_;
    using MemoryTree<HashingPolicy>::total_size_;
    std::vector<WrappedNullifierLeaf<HashingPolicy>> leaves_;
};

template <typename HashingPolicy>
NullifierMemoryTree<HashingPolicy>::NullifierMemoryTree(size_t depth, size_t initial_size)
    : MemoryTree<HashingPolicy>(depth)
{
    ASSERT(depth_ >= 1 && depth <= 32);
    ASSERT(initial_size > 0);
    total_size_ = 1UL << depth_;
    hashes_.resize(total_size_ * 2 - 2);

    // Build the entire tree and fill with 0 hashes.
    auto current = WrappedNullifierLeaf<HashingPolicy>(nullifier_leaf::zero()).hash();
    size_t layer_size = total_size_;
    for (size_t offset = 0; offset < hashes_.size(); offset += layer_size, layer_size /= 2) {
        for (size_t i = 0; i < layer_size; ++i) {
            hashes_[offset + i] = current;
        }
        current = HashingPolicy::hash_pair(current, current);
    }

    // Insert the initial leaves
    for (size_t i = 0; i < initial_size; i++) {
        auto initial_leaf =
            WrappedNullifierLeaf<HashingPolicy>(nullifier_leaf{ .value = i, .nextIndex = i + 1, .nextValue = i + 1 });
        leaves_.push_back(initial_leaf);
    }

    leaves_[initial_size - 1] = WrappedNullifierLeaf<HashingPolicy>(
        nullifier_leaf{ .value = leaves_[initial_size - 1].unwrap().value, .nextIndex = 0, .nextValue = 0 });

    for (size_t i = 0; i < initial_size; ++i) {
        update_element(i, leaves_[i].hash());
    }
}

template <typename HashingPolicy> fr_hash_path NullifierMemoryTree<HashingPolicy>::update_element(fr const& value)
{
    // Find the leaf with the value closest and less than `value`

    // If value is 0 we simply append 0 a null NullifierLeaf to the tree
    fr_hash_path hash_path;
    if (value == 0) {
        auto zero_leaf = WrappedNullifierLeaf<HashingPolicy>::zero();
        hash_path = get_hash_path(leaves_.size() - 1);
        leaves_.push_back(zero_leaf);
        update_element(leaves_.size() - 1, zero_leaf.hash());
        return hash_path;
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

    hash_path = get_hash_path(current);
    // Update the old leaf in the tree
    auto old_leaf_hash = HashingPolicy::hash(current_leaf.get_hash_inputs());
    size_t old_leaf_index = current;
    auto root = update_element(old_leaf_index, old_leaf_hash);

    // Insert the new leaf in the tree
    auto new_leaf_hash = HashingPolicy::hash(new_leaf.get_hash_inputs());
    size_t new_leaf_index = is_already_present ? old_leaf_index : leaves_.size() - 1;
    root = update_element(new_leaf_index, new_leaf_hash);

    return hash_path;
}

} // namespace bb::crypto::merkle_tree