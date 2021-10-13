#include "memory_tree.hpp"
#include "hash.hpp"

namespace plonk {
namespace stdlib {
namespace merkle_tree {

MemoryTree::MemoryTree(size_t depth)
    : depth_(depth)
{
    ASSERT(depth_ >= 1 && depth <= 20);
    total_size_ = 1UL << depth_;
    hashes_.resize(total_size_ * 2 - 2);

    // Build the entire tree.
    auto current = fr::neg_one();
    size_t layer_size = total_size_;
    for (size_t offset = 0; offset < hashes_.size(); offset += layer_size, layer_size /= 2) {
        // std::cerr << "zero: " << current << std::endl;
        for (size_t i = 0; i < layer_size; ++i) {
            hashes_[offset + i] = current;
        }
        current = compress_native(current, current);
    }

    // std::cerr << "root: " << current << std::endl;
    root_ = current;
}

fr_hash_path MemoryTree::get_hash_path(size_t index)
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

fr MemoryTree::update_element(size_t index, fr const& value)
{
    size_t offset = 0;
    size_t layer_size = total_size_;
    fr current = value == 0 ? fr::neg_one() : value;
    for (size_t i = 0; i < depth_; ++i) {
        hashes_[offset + index] = current;
        index &= (~0ULL) - 1;
        current = compress_native(hashes_[offset + index], hashes_[offset + index + 1]);
        offset += layer_size;
        layer_size /= 2;
        index /= 2;
    }
    root_ = current;
    return root_;
}

} // namespace merkle_tree
} // namespace stdlib
} // namespace plonk