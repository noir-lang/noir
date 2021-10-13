#include "merkle_tree.hpp"
#include "hash.hpp"
#include "leveldb_store.hpp"
#include "memory_store.hpp"
#include <common/net.hpp>
#include <iostream>
#include <numeric/bitop/count_leading_zeros.hpp>
#include <numeric/bitop/keep_n_lsb.hpp>
#include <numeric/uint128/uint128.hpp>
#include <sstream>

namespace plonk {
namespace stdlib {
namespace merkle_tree {

using namespace barretenberg;

template <typename T> inline bool bit_set(T const& index, size_t i)
{
    return bool((index >> i) & 0x1);
}

template <typename Store>
MerkleTree<Store>::MerkleTree(Store& store, size_t depth, uint8_t tree_id)
    : store_(store)
    , depth_(depth)
    , tree_id_(tree_id)
{
    ASSERT(depth_ >= 1 && depth <= 256);
    zero_hashes_.resize(depth);

    // Compute the zero values at each layer.
    auto current = fr::neg_one();
    for (size_t i = 0; i < depth; ++i) {
        zero_hashes_[i] = current;
        // std::cerr << "zero hash level " << i << ": " << current << std::endl;
        current = compress_native(current, current);
    }
}

template <typename Store>
MerkleTree<Store>::MerkleTree(MerkleTree&& other)
    : store_(other.store_)
    , zero_hashes_(std::move(other.zero_hashes_))
    , depth_(other.depth_)
    , tree_id_(other.tree_id_)
{}

template <typename Store> MerkleTree<Store>::~MerkleTree() {}

template <typename Store> fr MerkleTree<Store>::root() const
{
    std::vector<uint8_t> root;
    std::vector<uint8_t> key = { tree_id_ };
    bool status = store_.get(key, root);
    return status ? from_buffer<fr>(root) : compress_native(zero_hashes_.back(), zero_hashes_.back());
}

template <typename Store> typename MerkleTree<Store>::index_t MerkleTree<Store>::size() const
{
    std::vector<uint8_t> size_buf;
    std::vector<uint8_t> key = { tree_id_ };
    bool status = store_.get(key, size_buf);
    return status ? from_buffer<index_t>(size_buf, 32) : 0;
}

template <typename Store> fr_hash_path MerkleTree<Store>::get_hash_path(index_t index)
{
    fr_hash_path path(depth_);

    std::vector<uint8_t> data;
    bool status = store_.get(root().to_buffer(), data);

    for (size_t i = depth_ - 1; i < depth_; --i) {
        if (!status) {
            // This is an empty subtree. Fill in zero value.
            path[i] = std::make_pair(zero_hashes_[i], zero_hashes_[i]);
            continue;
        }

        if (data.size() == 64) {
            // This is a regular node with left and right trees. Descend according to index path.
            auto left = from_buffer<fr>(data, 0);
            auto right = from_buffer<fr>(data, 32);
            path[i] = std::make_pair(left, right);
            bool is_right = bit_set(index, i);
            auto it = data.data() + (is_right ? 32 : 0);
            status = store_.get(std::vector<uint8_t>(it, it + 32), data);
        } else {
            // This is a stump. The hash path can be fully restored from this node.
            fr current = from_buffer<fr>(data, 0);
            index_t element_index = from_buffer<index_t>(data, 32);
            index_t subtree_index = numeric::keep_n_lsb(index, i + 1);
            index_t diff = element_index ^ subtree_index;

            // std::cerr << "ghp hit stump height:" << i << " element_index:" << (uint64_t)element_index
            //           << " index:" << (uint64_t)index << " diff:" << (uint64_t)diff << std::endl;

            if (diff < 2) {
                // Requesting path to either the same element in the stump, or it's partner element.
                // Starting at the bottom of the tree, compute the remaining path pairs.
                for (size_t j = 0; j <= i; ++j) {
                    bool is_right = bit_set(element_index, j);
                    if (is_right) {
                        path[j] = std::make_pair(zero_hashes_[j], current);
                    } else {
                        path[j] = std::make_pair(current, zero_hashes_[j]);
                    }
                    current = compress_native(path[j].first, path[j].second);
                }
            } else {
                // Requesting path to a different, indepenent element.
                // We know that this element exits in an empty subtree, of height determined by the common bits in the
                // stumps index and the requested index.
                size_t common_bits = numeric::count_leading_zeros(diff);
                size_t ignored_bits = sizeof(index_t) * 8 - i;
                size_t common_height = i - (common_bits - ignored_bits) - 1;

                // std::cerr << "ghp diff:" << (uint64_t)diff << " ch:" << common_height << std::endl;

                for (size_t j = 0; j < common_height; ++j) {
                    path[j] = std::make_pair(zero_hashes_[j], zero_hashes_[j]);
                }
                current = compute_zero_path_hash(common_height, element_index, current);
                for (size_t j = common_height; j <= i; ++j) {
                    bool is_right = bit_set(element_index, j);
                    if (is_right) {
                        path[j] = std::make_pair(zero_hashes_[j], current);
                    } else {
                        path[j] = std::make_pair(current, zero_hashes_[j]);
                    }
                    current = compress_native(path[j].first, path[j].second);
                }
            }
            break;
        }
    }

    return path;
}

template <typename Store> fr MerkleTree<Store>::update_element(index_t index, fr const& value)
{
    auto leaf = (value == 0) ? fr::neg_one() : value;
    // std::cerr << "update_element: " << (uint64_t)index << std::endl;
    using serialize::write;
    std::vector<uint8_t> leaf_key;
    write(leaf_key, tree_id_);
    write(leaf_key, index);
    store_.put(leaf_key, to_buffer(leaf));

    auto r = update_element(root(), leaf, index, depth_);

    std::vector<uint8_t> meta_key = { tree_id_ };
    std::vector<uint8_t> meta_buf;
    write(meta_buf, r);
    write(meta_buf, index + 1);
    store_.put(meta_key, meta_buf);

    return r;
}

template <typename Store> fr MerkleTree<Store>::binary_put(index_t a_index, fr const& a, fr const& b, size_t height)
{
    bool a_is_right = bit_set(a_index, height - 1);
    auto left = a_is_right ? b : a;
    auto right = a_is_right ? a : b;
    auto key = compress_native(left, right);
    put(key, left, right);
    // std::cerr << "BINARY PUT height: " << height << " key:" << key << " left:" << left << " right:" << right
    //<< std::endl;
    return key;
}

template <typename Store>
fr MerkleTree<Store>::fork_stump(
    fr const& value1, index_t index1, fr const& value2, index_t index2, size_t height, size_t common_height)
{
    if (height == common_height) {
        if (height == 1) {
            // std::cerr << "Stump forked into leaves." << std::endl;
            index1 = numeric::keep_n_lsb(index1, 1);
            index2 = numeric::keep_n_lsb(index2, 1);
            return binary_put(index1, value1, value2, height);
        } else {
            size_t stump_height = height - 1;
            index_t stump1_index = numeric::keep_n_lsb(index1, stump_height);
            index_t stump2_index = numeric::keep_n_lsb(index2, stump_height);
            // std::cerr << "Stump forked into two at height " << stump_height << " index1 " << (uint64_t)index1
            //           << " index2 " << (uint64_t)index2 << std::endl;
            fr stump1_hash = compute_zero_path_hash(stump_height, stump1_index, value1);
            fr stump2_hash = compute_zero_path_hash(stump_height, stump2_index, value2);
            put_stump(stump1_hash, stump1_index, value1);
            put_stump(stump2_hash, stump2_index, value2);
            return binary_put(index1, stump1_hash, stump2_hash, height);
        }
    } else {
        auto new_root = fork_stump(value1, index1, value2, index2, height - 1, common_height);
        // std::cerr << "Stump branch hash at " << height << " " << new_root << " " << zero_hashes_[height] <<
        // std::endl;
        return binary_put(index1, new_root, zero_hashes_[height - 1], height);
    }
}

template <typename Store>
fr MerkleTree<Store>::update_element(fr const& root, fr const& value, index_t index, size_t height)
{
    // std::cerr << "update_element root:" << root << " value:" << value << " index:" << (uint64_t)index
    //           << " height:" << height << std::endl;
    if (height == 0) {
        return value;
    }

    std::vector<uint8_t> data;
    auto status = store_.get(root.to_buffer(), data);

    if (!status) {
        // std::cerr << "Adding new stump at height " << height << std::endl;
        fr key = compute_zero_path_hash(height, index, value);
        put_stump(key, index, value);
        return key;
    }

    // std::cerr << "got data of size " << data.size() << std::endl;
    if (data.size() != 64) {
        // We've come across a stump.
        index_t existing_index = from_buffer<index_t>(data, 32);

        if (existing_index == index) {
            // We are updating the stumps element. Easy update.
            // std::cerr << "Updating existing stump element at index " << (uint64_t)index << std::endl;
            fr new_hash = compute_zero_path_hash(height, index, value);
            put_stump(new_hash, existing_index, value);
            return new_hash;
        }

        fr existing_value = from_buffer<fr>(data, 0);
        size_t common_bits = numeric::count_leading_zeros(existing_index ^ index);
        size_t ignored_bits = sizeof(index_t) * 8 - height;
        size_t common_height = height - (common_bits - ignored_bits);
        // std::cerr << height << " common_bits:" << common_bits << " ignored_bits:" << ignored_bits
        //           << " existing_index:" << (uint64_t)existing_index << " index:" << (uint64_t)index
        //           << " common_height:" << common_height << std::endl;

        return fork_stump(existing_value, existing_index, value, index, height, common_height);
    } else {
        bool is_right = bit_set(index, height - 1);
        // std::cerr << "Normal node is_right:" << is_right << std::endl;
        fr subtree_root = from_buffer<fr>(data, is_right ? 32 : 0);
        subtree_root = update_element(subtree_root, value, numeric::keep_n_lsb(index, height - 1), height - 1);
        auto left = from_buffer<fr>(data, 0);
        auto right = from_buffer<fr>(data, 32);
        if (is_right) {
            right = subtree_root;
        } else {
            left = subtree_root;
        }
        auto new_root = compress_native(left, right);
        put(new_root, left, right);
        // TODO: Perhaps delete old node?
        return new_root;
    }
}

/**
 * Computes the root hash of a tree of `height`, that is empty other than `value` at `index`.
 */
template <typename Store> fr MerkleTree<Store>::compute_zero_path_hash(size_t height, index_t index, fr const& value)
{
    fr current = value;
    for (size_t i = 0; i < height; ++i) {
        bool is_right = bit_set(index, i);
        fr left, right;
        if (is_right) {
            left = zero_hashes_[i];
            right = current;
        } else {
            right = zero_hashes_[i];
            left = current;
        }
        current = compress_native(is_right ? zero_hashes_[i] : current, is_right ? current : zero_hashes_[i]);
    }
    return current;
}

template <typename Store> void MerkleTree<Store>::put(fr const& key, fr const& left, fr const& right)
{
    std::vector<uint8_t> value;
    write(value, left);
    write(value, right);
    store_.put(key.to_buffer(), value);
    // std::cerr << "PUT key:" << key << " left:" << left << " right:" << right << std::endl;
}

template <typename Store> void MerkleTree<Store>::put_stump(fr const& key, index_t index, fr const& value)
{
    std::vector<uint8_t> buf;
    write(buf, value);
    write(buf, index);
    // Add an additional byte, to signify we are a stump.
    write(buf, true);
    store_.put(key.to_buffer(), buf);
    // std::cerr << "PUT STUMP key:" << key << " index:" << (uint64_t)index << " value:" << value << std::endl;
}

#ifndef __wasm__
template class MerkleTree<LevelDbStore>;
#endif
template class MerkleTree<MemoryStore>;

} // namespace merkle_tree
} // namespace stdlib
} // namespace plonk