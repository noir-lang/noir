#pragma once
#include "../hash_path.hpp"

namespace bb::crypto::merkle_tree {

using namespace bb;

typedef uint256_t index_t;

/**
 * @brief Implements a simple append-only merkle tree
 * Accepts template argument of the type of store backing the tree and the hashing policy
 *
 */
template <typename Store, typename HashingPolicy> class AppendOnlyTree {
  public:
    AppendOnlyTree(Store& store, size_t depth, uint8_t tree_id = 0);
    AppendOnlyTree(AppendOnlyTree const& other) = delete;
    AppendOnlyTree(AppendOnlyTree&& other) = delete;
    virtual ~AppendOnlyTree();

    /**
     * @brief Adds a single value to the end of the tree
     */
    virtual fr add_value(const fr& value);

    /**
     * @brief Adds the given set of values to the end of the tree
     */
    virtual fr add_values(const std::vector<fr>& values);

    /**
     * @brief Returns the index of the right-most populated leaf in the tree
     */
    index_t size() const;

    /**
     * @brief Returns the root of the tree
     */
    fr root() const;

    /**
     * @brief Returns the depth of the tree
     */
    size_t depth() const;

    /**
     * @brief Returns the hash path from the leaf at the given index to the root
     */
    fr_hash_path get_hash_path(const index_t& index) const;

  protected:
    fr get_element_or_zero(size_t level, const index_t& index) const;

    void write_node(size_t level, const index_t& index, const fr& value);
    std::pair<bool, fr> read_node(size_t level, const index_t& index) const;

    Store& store_;
    size_t depth_;
    uint8_t tree_id_;
    std::vector<fr> zero_hashes_;
    fr root_;
    index_t size_;
};

template <typename Store, typename HashingPolicy>
AppendOnlyTree<Store, HashingPolicy>::AppendOnlyTree(Store& store, size_t depth, uint8_t tree_id)
    : store_(store)
    , depth_(depth)
    , tree_id_(tree_id)
{
    zero_hashes_.resize(depth + 1);

    // Create the zero hashes for the tree
    auto current = HashingPolicy::zero_hash();
    for (size_t i = depth; i > 0; --i) {
        zero_hashes_[i] = current;
        current = HashingPolicy::hash_pair(current, current);
    }
    zero_hashes_[0] = current;
    root_ = current;
}

template <typename Store, typename HashingPolicy> AppendOnlyTree<Store, HashingPolicy>::~AppendOnlyTree() {}

template <typename Store, typename HashingPolicy> index_t AppendOnlyTree<Store, HashingPolicy>::size() const
{
    return size_;
}

template <typename Store, typename HashingPolicy> fr AppendOnlyTree<Store, HashingPolicy>::root() const
{
    return root_;
}

template <typename Store, typename HashingPolicy> size_t AppendOnlyTree<Store, HashingPolicy>::depth() const
{
    return depth_;
}

template <typename Store, typename HashingPolicy>
fr_hash_path AppendOnlyTree<Store, HashingPolicy>::get_hash_path(const index_t& index) const
{
    fr_hash_path path;
    index_t current_index = index;

    for (size_t level = depth_; level > 0; --level) {
        bool is_right = bool(current_index & 0x01);
        fr right_value =
            is_right ? get_element_or_zero(level, current_index) : get_element_or_zero(level, current_index + 1);
        fr left_value =
            is_right ? get_element_or_zero(level, current_index - 1) : get_element_or_zero(level, current_index);
        path.push_back(std::make_pair(left_value, right_value));
        current_index >>= 1;
    }
    return path;
}

template <typename Store, typename HashingPolicy> fr AppendOnlyTree<Store, HashingPolicy>::add_value(const fr& value)
{
    return add_values(std::vector<fr>{ value });
}

template <typename Store, typename HashingPolicy>
fr AppendOnlyTree<Store, HashingPolicy>::add_values(const std::vector<fr>& values)
{
    index_t index = size();
    size_t number_to_insert = values.size();
    size_t level = depth_;
    std::vector<fr> hashes = values;

    // Add the values at the leaf nodes of the tree
    for (size_t i = 0; i < number_to_insert; ++i) {
        write_node(level, index + i, hashes[i]);
    }

    // Hash the values as a sub tree and insert them
    while (number_to_insert > 1) {
        number_to_insert >>= 1;
        index >>= 1;
        --level;
        for (size_t i = 0; i < number_to_insert; ++i) {
            hashes[i] = HashingPolicy::hash_pair(hashes[i * 2], hashes[i * 2 + 1]);
            write_node(level, index + i, hashes[i]);
        }
    }

    // Hash from the root of the sub-tree to the root of the overall tree
    fr new_hash = hashes[0];
    while (level > 0) {
        bool is_right = bool(index & 0x01);
        fr left_hash = is_right ? get_element_or_zero(level, index - 1) : new_hash;
        fr right_hash = is_right ? new_hash : get_element_or_zero(level, index + 1);
        new_hash = HashingPolicy::hash_pair(left_hash, right_hash);
        index >>= 1;
        --level;
        write_node(level, index, new_hash);
    }
    size_ += values.size();
    root_ = new_hash;
    return root_;
}

template <typename Store, typename HashingPolicy>
fr AppendOnlyTree<Store, HashingPolicy>::get_element_or_zero(size_t level, const index_t& index) const
{
    const std::pair<bool, fr> read_data = read_node(level, index);
    if (read_data.first) {
        return read_data.second;
    }
    ASSERT(level > 0 && level < zero_hashes_.size());
    return zero_hashes_[level];
}

template <typename Store, typename HashingPolicy>
void AppendOnlyTree<Store, HashingPolicy>::write_node(size_t level, const index_t& index, const fr& value)
{
    std::vector<uint8_t> buf;
    write(buf, value);
    store_.put(level, size_t(index), buf);
}

template <typename Store, typename HashingPolicy>
std::pair<bool, fr> AppendOnlyTree<Store, HashingPolicy>::read_node(size_t level, const index_t& index) const
{
    std::vector<uint8_t> buf;
    bool available = store_.get(level, size_t(index), buf);
    if (!available) {
        return std::make_pair(false, fr::zero());
    }
    fr value = from_buffer<fr>(buf, 0);
    return std::make_pair(true, value);
}

} // namespace bb::crypto::merkle_tree