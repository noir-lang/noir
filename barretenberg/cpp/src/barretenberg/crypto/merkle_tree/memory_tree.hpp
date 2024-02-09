#pragma once
#include "hash_path.hpp"

namespace bb::crypto::merkle_tree {

/**
 * A MemoryTree is structured as follows:
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
 */
template <typename HashingPolicy> class MemoryTree {
  public:
    MemoryTree(size_t depth);

    fr_hash_path get_hash_path(size_t index);

    fr_sibling_path get_sibling_path(size_t index);

    fr update_element(size_t index, fr const& value);

    fr root() const { return root_; }

  public:
    size_t depth_;
    size_t total_size_;
    bb::fr root_;
    std::vector<bb::fr> hashes_;
};

template <typename HashingPolicy>
MemoryTree<HashingPolicy>::MemoryTree(size_t depth)
    : depth_(depth)
{

    ASSERT(depth_ >= 1 && depth <= 20);
    total_size_ = 1UL << depth_;
    hashes_.resize(total_size_ * 2 - 2);

    // Build the entire tree.
    auto current = fr(0);
    size_t layer_size = total_size_;
    for (size_t offset = 0; offset < hashes_.size(); offset += layer_size, layer_size /= 2) {
        for (size_t i = 0; i < layer_size; ++i) {
            hashes_[offset + i] = current;
        }
        current = HashingPolicy::hash_pair(current, current);
    }

    root_ = current;
}

template <typename HashingPolicy> fr_hash_path MemoryTree<HashingPolicy>::get_hash_path(size_t index)
{
    fr_hash_path path(depth_);
    size_t offset = 0;
    size_t layer_size = total_size_;
    for (size_t i = 0; i < depth_; ++i) {
        index -= index & 0x1;
        path[i] = std::make_pair(hashes_[offset + index], hashes_[offset + index + 1]);
        offset += layer_size;
        layer_size >>= 1;
        index >>= 1;
    }
    return path;
}

template <typename HashingPolicy> fr_sibling_path MemoryTree<HashingPolicy>::get_sibling_path(size_t index)
{
    fr_sibling_path path(depth_);
    size_t offset = 0;
    size_t layer_size = total_size_;
    for (size_t i = 0; i < depth_; i++) {
        if (index % 2 == 0) {
            path[i] = hashes_[offset + index + 1];
        } else {
            path[i] = hashes_[offset + index - 1];
        }
        offset += layer_size;
        layer_size >>= 1;
        index >>= 1;
    }
    return path;
}

template <typename HashingPolicy> fr MemoryTree<HashingPolicy>::update_element(size_t index, fr const& value)
{
    size_t offset = 0;
    size_t layer_size = total_size_;
    fr current = value;
    for (size_t i = 0; i < depth_; ++i) {
        hashes_[offset + index] = current;
        index &= (~0ULL) - 1;
        current = HashingPolicy::hash_pair(hashes_[offset + index], hashes_[offset + index + 1]);
        offset += layer_size;
        layer_size >>= 1;
        index >>= 1;
    }
    root_ = current;
    return root_;
}

} // namespace bb::crypto::merkle_tree