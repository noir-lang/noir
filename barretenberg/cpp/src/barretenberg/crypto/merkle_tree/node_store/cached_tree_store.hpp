#pragma once
#include "./tree_meta.hpp"
#include "barretenberg/crypto/merkle_tree/indexed_tree/indexed_leaf.hpp"
#include "barretenberg/crypto/merkle_tree/lmdb_store/lmdb_store.hpp"
#include "barretenberg/crypto/merkle_tree/types.hpp"
#include "barretenberg/numeric/uint256/uint256.hpp"
#include "barretenberg/serialize/msgpack.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"
#include "msgpack/assert.hpp"
#include <cstdint>
#include <exception>
#include <memory>
#include <optional>
#include <unordered_map>
#include <utility>

namespace bb::crypto::merkle_tree {

/**
 * @brief Serves as a key-value node store for merkle trees. Caches all changes in memory before persisting them during
 * a 'commit' operation.
 * Manages the persisted store by seperating the key spaces as follows:
 * 1 byte key of 0: Tree meta data
 * 8 byte integers: The index of each leaf to the value of that leaf
 * 16 byte integers: Nodes in the tree, key value = ((2 ^ level) + index - 1)
 * 32 bytes integers: The value of the leaf (32 bytes) to the set of indices where the leaf exists in the tree.
 */
template <typename PersistedStore, typename LeafValueType> class CachedTreeStore {
  public:
    using PersistedStoreType = PersistedStore;
    using LeafType = LeafValueType;
    using IndexedLeafValueType = IndexedLeaf<LeafValueType>;
    using ReadTransaction = typename PersistedStore::ReadTransaction;
    using WriteTransaction = typename PersistedStore::WriteTransaction;
    using ReadTransactionPtr = std::unique_ptr<ReadTransaction>;
    using WriteTransactionPtr = std::unique_ptr<WriteTransaction>;

    CachedTreeStore(std::string name, uint32_t levels, PersistedStore& dataStore)
        : name(std::move(name))
        , depth(levels)
        , nodes(std::vector<std::unordered_map<index_t, std::vector<uint8_t>>>(
              depth + 1, std::unordered_map<index_t, std::vector<uint8_t>>()))
        , dataStore(dataStore)
    {
        initialise();
    }
    ~CachedTreeStore() = default;

    CachedTreeStore() = delete;
    CachedTreeStore(CachedTreeStore const& other) = delete;
    CachedTreeStore(CachedTreeStore const&& other) = delete;
    CachedTreeStore& operator=(CachedTreeStore const& other) = delete;
    CachedTreeStore& operator=(CachedTreeStore const&& other) = delete;

    /**
     * @brief Returns the index of the leaf with a value immediately lower than the value provided
     */
    std::pair<bool, index_t> find_low_value(const fr& new_leaf_key, bool includeUncommitted, ReadTransaction& tx) const;

    /**
     * @brief Returns the leaf at the provided index, if one exists
     */
    std::optional<IndexedLeafValueType> get_leaf(const index_t& index,
                                                 ReadTransaction& tx,
                                                 bool includeUncommitted) const;

    /**
     * @brief Adds the leaf at the given index, updates the leaf index if requested
     */
    void set_at_index(const index_t& index, const IndexedLeafValueType& leaf, bool add_to_index);

    /**
     * @brief Updates the leaf index
     */
    void update_index(const index_t& index, const fr& leaf);

    /**
     * @brief Writes the provided data at the given node coordinates. Only writes to uncommitted data.
     */
    void put_node(uint32_t level, index_t index, const std::vector<uint8_t>& data);

    /**
     * @brief Returns the data at the given node coordinates if available. Reads from uncommitted state if requested.
     */
    bool get_node(uint32_t level,
                  index_t index,
                  std::vector<uint8_t>& data,
                  ReadTransaction& transaction,
                  bool includeUncommitted) const;

    /**
     * @brief Writes the provided meta data to uncommitted state
     */
    void put_meta(const index_t& size, const bb::fr& root);

    /**
     * @brief Reads the tree meta data, including uncommitted data if requested
     */
    void get_meta(index_t& size, bb::fr& root, ReadTransaction& tx, bool includeUncommitted) const;

    /**
     * @brief Reads the extended tree meta data, including uncommitted data if requested
     */
    void get_full_meta(index_t& size,
                       bb::fr& root,
                       std::string& name,
                       uint32_t& depth,
                       ReadTransaction& tx,
                       bool includeUncommitted) const;

    /**
     * @brief Finds the index of the given leaf value in the tree if available. Includes uncommitted data if requested.
     */
    std::optional<index_t> find_leaf_index(const LeafValueType& leaf,
                                           ReadTransaction& tx,
                                           bool includeUncommitted) const;

    /**
     * @brief Finds the index of the given leaf value in the tree if available. Includes uncommitted data if requested.
     */
    std::optional<index_t> find_leaf_index_from(const LeafValueType& leaf,
                                                index_t start_index,
                                                ReadTransaction& tx,
                                                bool includeUncommitted) const;

    /**
     * @brief Commits the uncommitted data to the underlying store
     */
    void commit();

    /**
     * @brief Rolls back the uncommitted state
     */
    void rollback();

    /**
     * @brief Returns the name of the tree
     */
    std::string get_name() const { return name; }

    /**
     * @brief Returns a read transaction against the underlying store.
     */
    ReadTransactionPtr create_read_transaction() const { return dataStore.create_read_transaction(); }

  private:
    struct Indices {
        std::vector<index_t> indices;

        MSGPACK_FIELDS(indices);
    };

    std::string name;
    uint32_t depth;
    std::vector<std::unordered_map<index_t, std::vector<uint8_t>>> nodes;
    std::map<uint256_t, Indices> indices_;
    std::unordered_map<index_t, IndexedLeafValueType> leaves_;
    PersistedStore& dataStore;
    TreeMeta meta;

    void initialise();

    bool read_persisted_meta(TreeMeta& m, ReadTransaction& tx) const;

    void persist_meta(TreeMeta& m, WriteTransaction& tx);

    WriteTransactionPtr create_write_transaction() const { return dataStore.create_write_transaction(); }
};

template <typename PersistedStore, typename LeafValueType>
std::pair<bool, index_t> CachedTreeStore<PersistedStore, LeafValueType>::find_low_value(const fr& new_leaf_key,
                                                                                        bool includeUncommitted,
                                                                                        ReadTransaction& tx) const
{
    uint256_t new_value_as_number = uint256_t(new_leaf_key);
    std::vector<uint8_t> data;
    FrKeyType key(new_leaf_key);
    tx.get_value_or_previous(key, data);
    Indices committed;
    msgpack::unpack((const char*)data.data(), data.size()).get().convert(committed);
    auto db_index = committed.indices[0];
    uint256_t retrieved_value = key;
    if (!includeUncommitted || retrieved_value == new_value_as_number || indices_.empty()) {
        return std::make_pair(new_value_as_number == retrieved_value, db_index);
    }

    // At this stage, we have been asked to include uncommitted and the value was not exactly found in the db
    auto it = indices_.lower_bound(new_value_as_number);
    if (it == indices_.end()) {
        // there is no element >= the requested value.
        // decrement the iterator to get the value preceeding the requested value
        --it;
        // we need to return the larger of the db value or the cached value

        return std::make_pair(false, it->first > retrieved_value ? it->second.indices[0] : db_index);
    }

    if (it->first == uint256_t(new_value_as_number)) {
        // the value is already present and the iterator points to it
        return std::make_pair(true, it->second.indices[0]);
    }
    // the iterator points to the element immediately larger than the requested value
    // We need to return the highest value from
    // 1. The next lowest cached value, if there is one
    // 2. The value retrieved from the db
    if (it == indices_.begin()) {
        // No cached lower value, return the db index
        return std::make_pair(false, db_index);
    }
    --it;
    //  it now points to the value less than that requested
    return std::make_pair(false, it->first > retrieved_value ? it->second.indices[0] : db_index);
}

template <typename PersistedStore, typename LeafValueType>
std::optional<typename CachedTreeStore<PersistedStore, LeafValueType>::IndexedLeafValueType> CachedTreeStore<
    PersistedStore,
    LeafValueType>::get_leaf(const index_t& index, ReadTransaction& tx, bool includeUncommitted) const
{
    if (includeUncommitted) {
        typename std::unordered_map<index_t, IndexedLeafValueType>::const_iterator it = leaves_.find(index);
        if (it != leaves_.end()) {
            return it->second;
        }
    }
    LeafIndexKeyType key = index;
    std::vector<uint8_t> data;
    bool success = tx.get_value(key, data);
    if (success) {
        IndexedLeafValueType return_value;
        msgpack::unpack((const char*)data.data(), data.size()).get().convert(return_value);
        return return_value;
    }
    return std::nullopt;
}

template <typename PersistedStore, typename LeafValueType>
void CachedTreeStore<PersistedStore, LeafValueType>::set_at_index(const index_t& index,
                                                                  const IndexedLeafValueType& leaf,
                                                                  bool add_to_index)
{
    leaves_[index] = leaf;
    if (add_to_index) {
        auto it = indices_.find(uint256_t(leaf.value.get_key()));
        if (it == indices_.end()) {
            Indices indices;
            indices.indices.push_back(index);
            indices_[uint256_t(leaf.value.get_key())] = indices;
            return;
        }
        it->second.indices.push_back(index);
    }
}

template <typename PersistedStore, typename LeafValueType>
void CachedTreeStore<PersistedStore, LeafValueType>::update_index(const index_t& index, const fr& leaf)
{
    auto it = indices_.find(uint256_t(leaf));
    if (it == indices_.end()) {
        Indices indices;
        indices.indices.push_back(index);
        indices_[uint256_t(leaf)] = indices;
        return;
    }
    it->second.indices.push_back(index);
}

template <typename PersistedStore, typename LeafValueType>
std::optional<index_t> CachedTreeStore<PersistedStore, LeafValueType>::find_leaf_index(const LeafValueType& leaf,
                                                                                       ReadTransaction& tx,
                                                                                       bool includeUncommitted) const
{
    return find_leaf_index_from(leaf, 0, tx, includeUncommitted);
}

template <typename PersistedStore, typename LeafValueType>
std::optional<index_t> CachedTreeStore<PersistedStore, LeafValueType>::find_leaf_index_from(
    const LeafValueType& leaf, index_t start_index, ReadTransaction& tx, bool includeUncommitted) const
{
    Indices committed;
    std::optional<index_t> result = std::nullopt;
    FrKeyType key = leaf;
    std::vector<uint8_t> value;
    bool success = tx.get_value(key, value);
    if (success) {
        msgpack::unpack((const char*)value.data(), value.size()).get().convert(committed);
        if (!committed.indices.empty()) {
            for (size_t i = 0; i < committed.indices.size(); ++i) {
                index_t ind = committed.indices[i];
                if (ind < start_index) {
                    continue;
                }
                if (!result.has_value()) {
                    result = ind;
                    continue;
                }
                result = std::min(ind, result.value());
            }
        }
    }
    if (includeUncommitted) {
        auto it = indices_.find(uint256_t(leaf));
        if (it != indices_.end() && !it->second.indices.empty()) {
            for (size_t i = 0; i < it->second.indices.size(); ++i) {
                index_t ind = it->second.indices[i];
                if (ind < start_index) {
                    continue;
                }
                if (!result.has_value()) {
                    result = ind;
                    continue;
                }
                result = std::min(ind, result.value());
            }
        }
    }
    return result;
}

template <typename PersistedStore, typename LeafValueType>
void CachedTreeStore<PersistedStore, LeafValueType>::put_node(uint32_t level,
                                                              index_t index,
                                                              const std::vector<uint8_t>& data)
{
    nodes[level][index] = data;
}

template <typename PersistedStore, typename LeafValueType>
bool CachedTreeStore<PersistedStore, LeafValueType>::get_node(uint32_t level,
                                                              index_t index,
                                                              std::vector<uint8_t>& data,
                                                              ReadTransaction& transaction,
                                                              bool includeUncommitted) const
{
    if (includeUncommitted) {
        const auto& level_map = nodes[level];
        auto it = level_map.find(index);
        if (it != level_map.end()) {
            data = it->second;
            return true;
        }
    }
    return transaction.get_node(level, index, data);
}

template <typename PersistedStore, typename LeafValueType>
void CachedTreeStore<PersistedStore, LeafValueType>::put_meta(const index_t& size, const bb::fr& root)
{
    meta.root = root;
    meta.size = size;
}

template <typename PersistedStore, typename LeafValueType>
void CachedTreeStore<PersistedStore, LeafValueType>::get_meta(index_t& size,
                                                              bb::fr& root,
                                                              ReadTransaction& tx,
                                                              bool includeUncommitted) const
{
    if (includeUncommitted) {
        size = meta.size;
        root = meta.root;
        return;
    }
    TreeMeta m;
    read_persisted_meta(m, tx);
    size = m.size;
    root = m.root;
}

template <typename PersistedStore, typename LeafValueType>
void CachedTreeStore<PersistedStore, LeafValueType>::get_full_meta(
    index_t& size, bb::fr& root, std::string& name, uint32_t& depth, ReadTransaction& tx, bool includeUncommitted) const
{
    if (includeUncommitted) {
        size = meta.size;
        root = meta.root;
        name = meta.name;
        depth = meta.depth;
        return;
    }
    TreeMeta m;
    read_persisted_meta(m, tx);
    size = m.size;
    root = m.root;
    depth = m.depth;
    name = m.name;
}

template <typename PersistedStore, typename LeafValueType> void CachedTreeStore<PersistedStore, LeafValueType>::commit()
{
    {
        {
            ReadTransactionPtr tx = create_read_transaction();
            for (auto& idx : indices_) {
                std::vector<uint8_t> value;
                FrKeyType key = idx.first;
                bool success = tx->get_value(key, value);
                if (success) {
                    Indices indices;
                    msgpack::unpack((const char*)value.data(), value.size()).get().convert(indices);
                    idx.second.indices.insert(
                        idx.second.indices.begin(), indices.indices.begin(), indices.indices.end());
                }
            }
        }
        WriteTransactionPtr tx = create_write_transaction();
        try {
            for (uint32_t i = 1; i < nodes.size(); i++) {
                auto& level = nodes[i];
                for (auto& item : level) {
                    index_t index = item.first;
                    std::vector<uint8_t>& data = item.second;
                    tx->put_node(i, index, data);
                }
            }
            for (auto& idx : indices_) {
                msgpack::sbuffer buffer;
                msgpack::pack(buffer, idx.second);
                std::vector<uint8_t> encoded(buffer.data(), buffer.data() + buffer.size());
                FrKeyType key = idx.first;
                tx->put_value(key, encoded);
            }
            for (const auto& leaf : leaves_) {
                msgpack::sbuffer buffer;
                msgpack::pack(buffer, leaf.second);
                std::vector<uint8_t> value(buffer.data(), buffer.data() + buffer.size());
                LeafIndexKeyType key = leaf.first;
                tx->put_value(key, value);
            }
            persist_meta(meta, *tx);
            tx->commit();
        } catch (std::exception& e) {
            tx->try_abort();
            throw;
        }
    }
    rollback();
}

template <typename PersistedStore, typename LeafValueType>
void CachedTreeStore<PersistedStore, LeafValueType>::rollback()
{
    // Extract the committed meta data and destroy the cache
    {
        ReadTransactionPtr tx = create_read_transaction();
        read_persisted_meta(meta, *tx);
    }
    nodes = std::vector<std::unordered_map<index_t, std::vector<uint8_t>>>(
        depth + 1, std::unordered_map<index_t, std::vector<uint8_t>>());
    indices_ = std::map<uint256_t, Indices>();
    leaves_ = std::unordered_map<index_t, IndexedLeafValueType>();
}

template <typename PersistedStore, typename LeafValueType>
bool CachedTreeStore<PersistedStore, LeafValueType>::read_persisted_meta(TreeMeta& m, ReadTransaction& tx) const
{
    std::vector<uint8_t> data;
    bool success = tx.get_node(0, 0, data);
    if (success) {
        msgpack::unpack((const char*)data.data(), data.size()).get().convert(m);
    }
    return success;
}

template <typename PersistedStore, typename LeafValueType>
void CachedTreeStore<PersistedStore, LeafValueType>::persist_meta(TreeMeta& m, WriteTransaction& tx)
{
    msgpack::sbuffer buffer;
    msgpack::pack(buffer, m);
    std::vector<uint8_t> encoded(buffer.data(), buffer.data() + buffer.size());
    tx.put_node(0, 0, encoded);
}

template <typename PersistedStore, typename LeafValueType>
void CachedTreeStore<PersistedStore, LeafValueType>::initialise()
{
    // Read the persisted meta data, if the name or depth of the tree is not consistent with what was provided during
    // construction then we throw
    std::vector<uint8_t> data;
    {
        ReadTransactionPtr tx = create_read_transaction();
        bool success = read_persisted_meta(meta, *tx);
        if (success) {
            if (name == meta.name && depth == meta.depth) {
                return;
            }
            throw std::runtime_error("Invalid tree meta data");
        }
    }

    // No meta data available. Write the initial state down
    meta.name = name;
    meta.size = 0;
    meta.depth = depth;
    WriteTransactionPtr tx = create_write_transaction();
    try {
        persist_meta(meta, *tx);
        tx->commit();
    } catch (std::exception& e) {
        tx->try_abort();
        throw e;
    }
}

} // namespace bb::crypto::merkle_tree
