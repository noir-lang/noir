#pragma once
#include "../hash_path.hpp"
#include "../node_store//tree_meta.hpp"
#include "../response.hpp"
#include "../types.hpp"
#include "barretenberg/common/thread_pool.hpp"
#include "barretenberg/crypto/merkle_tree/indexed_tree/indexed_leaf.hpp"
#include "barretenberg/numeric/bitop/pow.hpp"
#include <exception>
#include <functional>
#include <iostream>
#include <memory>
#include <stdexcept>
#include <utility>

namespace bb::crypto::merkle_tree {

using namespace bb;

/**
 * @brief Implements a simple append-only merkle tree
 * All methods are asynchronous unless specified as otherwise
 * Accepts template arguments of the type of store backing the tree and the hashing policy
 * Accepts the store as an argument on construction as well as a thread pool instance
 * Asynchronous methods are exeucted on the provided thread pool
 *
 */
template <typename Store, typename HashingPolicy> class AppendOnlyTree {
  public:
    using StoreType = Store;

    // Asynchronous methods accept these callback function types as arguments
    using AppendCompletionCallback = std::function<void(const TypedResponse<AddDataResponse>&)>;
    using MetaDataCallback = std::function<void(const TypedResponse<TreeMetaResponse>&)>;
    using HashPathCallback = std::function<void(const TypedResponse<GetSiblingPathResponse>&)>;
    using FindLeafCallback = std::function<void(const TypedResponse<FindLeafIndexResponse>&)>;
    using GetLeafCallback = std::function<void(const TypedResponse<GetLeafResponse>&)>;
    using CommitCallback = std::function<void(const Response&)>;
    using RollbackCallback = std::function<void(const Response&)>;

    // Only construct from provided store and thread pool, no copies or moves
    AppendOnlyTree(Store& store, ThreadPool& workers);
    AppendOnlyTree(AppendOnlyTree const& other) = delete;
    AppendOnlyTree(AppendOnlyTree&& other) = delete;
    AppendOnlyTree& operator=(AppendOnlyTree const& other) = delete;
    AppendOnlyTree& operator=(AppendOnlyTree const&& other) = delete;
    virtual ~AppendOnlyTree() = default;

    /**
     * @brief Adds a single value to the end of the tree
     * @param value The value to be added
     * @param on_completion Callback to be called on completion
     */
    virtual void add_value(const fr& value, const AppendCompletionCallback& on_completion);

    /**
     * @brief Adds the given set of values to the end of the tree
     * @param values The values to be added
     * @param on_completion Callback to be called on completion
     */
    virtual void add_values(const std::vector<fr>& values, const AppendCompletionCallback& on_completion);

    /**
     * @brief Returns the sibling path from the leaf at the given index to the root
     * @param index The index at which to read the sibling path
     * @param on_completion Callback to be called on completion
     * @param includeUncommitted Whether to include uncommitted changes
     */
    void get_sibling_path(const index_t& index, const HashPathCallback& on_completion, bool includeUncommitted) const;

    /**
     * @brief Get the subtree sibling path object
     *
     * @param subtree_depth The depth of the subtree
     * @param on_completion Callback to be called on completion
     * @param includeUncommitted Whether to include uncommitted changes
     */
    void get_subtree_sibling_path(uint32_t subtree_depth,
                                  const HashPathCallback& on_completion,
                                  bool includeUncommitted) const;

    /**
     * @brief Get the subtree sibling path object to a leaf
     *
     * @param leaf_index The depth of the subtree
     * @param subtree_depth The depth of the subtree
     * @param on_completion Callback to be called on completion
     * @param includeUncommitted Whether to include uncommitted changes
     */
    void get_subtree_sibling_path(index_t leaf_index,
                                  uint32_t subtree_depth,
                                  const HashPathCallback& on_completion,
                                  bool includeUncommitted) const;

    /**
     * @brief Returns the tree meta data
     * @param includeUncommitted Whether to include uncommitted changes
     * @param on_completion Callback to be called on completion
     */
    void get_meta_data(bool includeUncommitted, const MetaDataCallback& on_completion) const;

    /**
     * @brief Returns the leaf value at the provided index
     * @param index The index of the leaf to be retrieved
     * @param includeUncommitted Whether to include uncommitted changes
     * @param on_completion Callback to be called on completion
     */
    void get_leaf(const index_t& index, bool includeUncommitted, const GetLeafCallback& completion) const;

    /**
     * @brief Returns the index of the provided leaf in the tree
     */
    void find_leaf_index(const fr& leaf, bool includeUncommitted, const FindLeafCallback& on_completion) const;

    /**
     * @brief Returns the index of the provided leaf in the tree only if it exists after the index value provided
     */
    void find_leaf_index_from(const fr& leaf,
                              index_t start_index,
                              bool includeUncommitted,
                              const FindLeafCallback& on_completion) const;

    /**
     * @brief Commit the tree to the backing store
     */
    void commit(const CommitCallback& on_completion);

    /**
     * @brief Rollback the uncommitted changes
     */
    void rollback(const RollbackCallback& on_completion);

    /**
     * @brief Synchronous method to retrieve the depth of the tree
     */
    uint32_t depth() const { return depth_; }

  protected:
    using ReadTransaction = typename Store::ReadTransaction;
    using ReadTransactionPtr = typename Store::ReadTransactionPtr;
    fr get_element_or_zero(uint32_t level, const index_t& index, ReadTransaction& tx, bool includeUncommitted) const;

    void write_node(uint32_t level, const index_t& index, const fr& value);
    std::pair<bool, fr> read_node(uint32_t level,
                                  const index_t& index,
                                  ReadTransaction& tx,
                                  bool includeUncommitted) const;

    void add_values_internal(std::shared_ptr<std::vector<fr>> values,
                             fr& new_root,
                             index_t& new_size,
                             bool update_index);

    void add_values_internal(const std::vector<fr>& values,
                             const AppendCompletionCallback& on_completion,
                             bool update_index);

    fr_sibling_path get_subtree_sibling_path_internal(index_t leaf_index,
                                                      uint32_t subtree_depth,
                                                      ReadTransaction& tx,
                                                      bool includeUncommitted) const;

    Store& store_;
    uint32_t depth_;
    std::string name_;
    uint64_t max_size_;
    std::vector<fr> zero_hashes_;
    ThreadPool& workers_;
};

template <typename Store, typename HashingPolicy>
AppendOnlyTree<Store, HashingPolicy>::AppendOnlyTree(Store& store, ThreadPool& workers)
    : store_(store)
    , workers_(workers)
{
    index_t stored_size = 0;
    bb::fr stored_root = fr::zero();
    {
        // start by reading the meta data from the backing store
        ReadTransactionPtr tx = store_.create_read_transaction();
        store_.get_full_meta(stored_size, stored_root, name_, depth_, *tx, false);
    }
    zero_hashes_.resize(depth_ + 1);

    // Create the zero hashes for the tree
    auto current = HashingPolicy::zero_hash();
    for (size_t i = depth_; i > 0; --i) {
        zero_hashes_[i] = current;
        current = HashingPolicy::hash_pair(current, current);
    }
    zero_hashes_[0] = current;

    if (stored_size == 0) {
        // if the tree is empty then we want to write the initial root
        store_.put_meta(0, current);
        store_.commit();
    }
    max_size_ = numeric::pow64(2, depth_);
}

template <typename Store, typename HashingPolicy>
void AppendOnlyTree<Store, HashingPolicy>::get_meta_data(bool includeUncommitted,
                                                         const MetaDataCallback& on_completion) const
{
    auto job = [=, this]() {
        execute_and_report<TreeMetaResponse>(
            [=, this](TypedResponse<TreeMetaResponse>& response) {
                ReadTransactionPtr tx = store_.create_read_transaction();
                store_.get_meta(response.inner.size, response.inner.root, *tx, includeUncommitted);
                response.inner.depth = depth_;
            },
            on_completion);
    };
    workers_.enqueue(job);
}

template <typename Store, typename HashingPolicy>
void AppendOnlyTree<Store, HashingPolicy>::get_sibling_path(const index_t& index,
                                                            const HashPathCallback& on_completion,
                                                            bool includeUncommitted) const
{
    auto job = [=, this]() {
        execute_and_report<GetSiblingPathResponse>(
            [=, this](TypedResponse<GetSiblingPathResponse>& response) {
                index_t current_index = index;
                ReadTransactionPtr tx = store_.create_read_transaction();
                for (uint32_t level = depth_; level > 0; --level) {
                    bool is_right = static_cast<bool>(current_index & 0x01);
                    fr sibling = is_right ? get_element_or_zero(level, current_index - 1, *tx, includeUncommitted)
                                          : get_element_or_zero(level, current_index + 1, *tx, includeUncommitted);
                    response.inner.path.emplace_back(sibling);
                    current_index >>= 1;
                }
            },
            on_completion);
    };
    workers_.enqueue(job);
}

template <typename Store, typename HashingPolicy>
void AppendOnlyTree<Store, HashingPolicy>::get_subtree_sibling_path(const uint32_t subtree_depth,
                                                                    const HashPathCallback& on_completion,
                                                                    bool includeUncommitted) const
{
    auto job = [=, this]() {
        execute_and_report<GetSiblingPathResponse>(
            [=, this](TypedResponse<GetSiblingPathResponse>& response) {
                ReadTransactionPtr tx = store_.create_read_transaction();
                index_t index_of_next_leaf = 0;
                bb::fr root;
                store_.get_meta(index_of_next_leaf, root, *tx, includeUncommitted);
                response.inner.path =
                    get_subtree_sibling_path_internal(index_of_next_leaf, subtree_depth, *tx, includeUncommitted);
            },
            on_completion);
    };
    workers_.enqueue(job);
}

template <typename Store, typename HashingPolicy>
void AppendOnlyTree<Store, HashingPolicy>::get_subtree_sibling_path(const index_t leaf_index,
                                                                    const uint32_t subtree_depth,
                                                                    const HashPathCallback& on_completion,
                                                                    bool includeUncommitted) const
{
    auto job = [=, this]() {
        execute_and_report<GetSiblingPathResponse>(
            [=, this](TypedResponse<GetSiblingPathResponse>& response) {
                ReadTransactionPtr tx = store_.create_read_transaction();
                response.inner.path =
                    get_subtree_sibling_path_internal(leaf_index, subtree_depth, *tx, includeUncommitted);
            },
            on_completion);
    };
    workers_.enqueue(job);
}

template <typename Store, typename HashingPolicy>
fr_sibling_path AppendOnlyTree<Store, HashingPolicy>::get_subtree_sibling_path_internal(const index_t leaf_index,
                                                                                        const uint32_t subtree_depth,
                                                                                        ReadTransaction& tx,
                                                                                        bool includeUncommitted) const
{
    // skip the first levels, all the way to the subtree_root
    index_t current_index = leaf_index >> subtree_depth;
    fr_sibling_path path;
    path.reserve(depth_ - subtree_depth);

    for (uint32_t level = depth_ - subtree_depth; level > 0; --level) {
        bool is_right = static_cast<bool>(current_index & 0x01);
        fr sibling = is_right ? get_element_or_zero(level, current_index - 1, tx, includeUncommitted)
                              : get_element_or_zero(level, current_index + 1, tx, includeUncommitted);
        path.emplace_back(sibling);
        current_index >>= 1;
    }

    return path;
}

template <typename Store, typename HashingPolicy>
void AppendOnlyTree<Store, HashingPolicy>::get_leaf(const index_t& index,
                                                    bool includeUncommitted,
                                                    const GetLeafCallback& on_completion) const
{
    auto job = [=, this]() {
        execute_and_report<GetLeafResponse>(
            [=, this](TypedResponse<GetLeafResponse>& response) {
                ReadTransactionPtr tx = store_.create_read_transaction();
                auto leaf = read_node(depth_, index, *tx, includeUncommitted);
                response.success = leaf.first;
                if (leaf.first) {
                    response.inner.leaf = leaf.second;
                }
            },
            on_completion);
    };
    workers_.enqueue(job);
}

template <typename Store, typename HashingPolicy>
void AppendOnlyTree<Store, HashingPolicy>::find_leaf_index(const fr& leaf,
                                                           bool includeUncommitted,
                                                           const FindLeafCallback& on_completion) const
{
    find_leaf_index_from(leaf, 0, includeUncommitted, on_completion);
}

template <typename Store, typename HashingPolicy>
void AppendOnlyTree<Store, HashingPolicy>::find_leaf_index_from(const fr& leaf,
                                                                index_t start_index,
                                                                bool includeUncommitted,
                                                                const FindLeafCallback& on_completion) const
{
    auto job = [=, this]() -> void {
        execute_and_report<FindLeafIndexResponse>(
            [=, this](TypedResponse<FindLeafIndexResponse>& response) {
                typename Store::ReadTransactionPtr tx = store_.create_read_transaction();
                std::optional<index_t> leaf_index =
                    store_.find_leaf_index_from(leaf, start_index, *tx, includeUncommitted);
                response.success = leaf_index.has_value();
                if (response.success) {
                    response.inner.leaf_index = leaf_index.value();
                }
            },
            on_completion);
    };
    workers_.enqueue(job);
}

template <typename Store, typename HashingPolicy>
void AppendOnlyTree<Store, HashingPolicy>::add_value(const fr& value, const AppendCompletionCallback& on_completion)
{
    add_values(std::vector<fr>{ value }, on_completion);
}

template <typename Store, typename HashingPolicy>
void AppendOnlyTree<Store, HashingPolicy>::add_values(const std::vector<fr>& values,
                                                      const AppendCompletionCallback& on_completion)
{
    add_values_internal(values, on_completion, true);
}

template <typename Store, typename HashingPolicy>
void AppendOnlyTree<Store, HashingPolicy>::add_values_internal(const std::vector<fr>& values,
                                                               const AppendCompletionCallback& on_completion,
                                                               bool update_index)
{
    std::shared_ptr<std::vector<fr>> hashes = std::make_shared<std::vector<fr>>(values);
    auto append_op = [=, this]() -> void {
        execute_and_report<AddDataResponse>(
            [=, this](TypedResponse<AddDataResponse>& response) {
                add_values_internal(hashes, response.inner.root, response.inner.size, update_index);
            },
            on_completion);
    };
    workers_.enqueue(append_op);
}

template <typename Store, typename HashingPolicy>
void AppendOnlyTree<Store, HashingPolicy>::commit(const CommitCallback& on_completion)
{
    auto job = [=, this]() { execute_and_report([=, this]() { store_.commit(); }, on_completion); };
    workers_.enqueue(job);
}

template <typename Store, typename HashingPolicy>
void AppendOnlyTree<Store, HashingPolicy>::rollback(const RollbackCallback& on_completion)
{
    auto job = [=, this]() { execute_and_report([=, this]() { store_.rollback(); }, on_completion); };
    workers_.enqueue(job);
}

template <typename Store, typename HashingPolicy>
void AppendOnlyTree<Store, HashingPolicy>::add_values_internal(std::shared_ptr<std::vector<fr>> values,
                                                               fr& new_root,
                                                               index_t& new_size,
                                                               bool update_index)
{

    uint32_t start_level = depth_;
    uint32_t level = start_level;
    std::vector<fr>& hashes_local = *values;
    auto number_to_insert = static_cast<uint32_t>(hashes_local.size());
    index_t start_size = 0;
    bb::fr root;

    typename Store::ReadTransactionPtr tx = store_.create_read_transaction();
    store_.get_meta(start_size, root, *tx, true);
    index_t index = start_size;
    new_size = start_size + number_to_insert;

    if (values->empty()) {
        return;
    }

    if (new_size > max_size_) {
        throw std::runtime_error("Tree is full");
    }

    // Add the values at the leaf nodes of the tree
    for (uint32_t i = 0; i < number_to_insert; ++i) {
        write_node(level, index + i, hashes_local[i]);
    }

    // If we have been told to add these leaves to the index then do so now
    if (update_index) {
        for (uint32_t i = 0; i < number_to_insert; ++i) {
            store_.update_index(index + i, hashes_local[i]);
        }
    }

    // Hash the values as a sub tree and insert them
    while (number_to_insert > 1) {
        number_to_insert >>= 1;
        index >>= 1;
        --level;
        for (uint32_t i = 0; i < number_to_insert; ++i) {
            hashes_local[i] = HashingPolicy::hash_pair(hashes_local[i * 2], hashes_local[i * 2 + 1]);
            write_node(level, index + i, hashes_local[i]);
        }
    }

    // Hash from the root of the sub-tree to the root of the overall tree
    fr new_hash = hashes_local[0];
    while (level > 0) {
        bool is_right = static_cast<bool>(index & 0x01);
        fr left_hash = is_right ? get_element_or_zero(level, index - 1, *tx, true) : new_hash;
        fr right_hash = is_right ? new_hash : get_element_or_zero(level, index + 1, *tx, true);
        new_hash = HashingPolicy::hash_pair(left_hash, right_hash);

        index >>= 1;
        --level;
        if (level > 0) {
            write_node(level, index, new_hash);
        }
    }
    new_root = new_hash;
    store_.put_meta(new_size, new_root);
}

// Retrieves the value at the given level and index or the 'zero' tree hash if not present
template <typename Store, typename HashingPolicy>
fr AppendOnlyTree<Store, HashingPolicy>::get_element_or_zero(uint32_t level,
                                                             const index_t& index,
                                                             ReadTransaction& tx,
                                                             bool includeUncommitted) const
{
    const std::pair<bool, fr> read_data = read_node(level, index, tx, includeUncommitted);
    if (read_data.first) {
        return read_data.second;
    }
    return zero_hashes_[level];
}

template <typename Store, typename HashingPolicy>
void AppendOnlyTree<Store, HashingPolicy>::write_node(uint32_t level, const index_t& index, const fr& value)
{
    std::vector<uint8_t> buf;
    write(buf, value);
    store_.put_node(level, index, buf);
}

template <typename Store, typename HashingPolicy>
std::pair<bool, fr> AppendOnlyTree<Store, HashingPolicy>::read_node(uint32_t level,
                                                                    const index_t& index,
                                                                    ReadTransaction& tx,
                                                                    bool includeUncommitted) const
{
    std::vector<uint8_t> buf;
    bool available = store_.get_node(level, index, buf, tx, includeUncommitted);
    if (!available) {
        return std::make_pair(false, fr::zero());
    }
    fr value = from_buffer<fr>(buf, 0);
    return std::make_pair(true, value);
}

} // namespace bb::crypto::merkle_tree
