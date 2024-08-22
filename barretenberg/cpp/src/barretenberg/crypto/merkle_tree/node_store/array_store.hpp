#pragma once
#include "./tree_meta.hpp"
#include "barretenberg/crypto/merkle_tree/types.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"
#include <memory>

namespace bb::crypto::merkle_tree {

class MockTransaction {
  public:
    using Ptr = std::unique_ptr<MockTransaction>;
    bool get_node(uint32_t, index_t, std::vector<uint8_t>&) const { return false; }

    template <typename T> void get_value_by_integer(T&, std::vector<uint8_t>&){};

    void get_value(std::vector<uint8_t>&, std::vector<uint8_t>&){};

    void put_node(uint32_t, index_t, const std::vector<uint8_t>&) {}

    template <typename T> void put_value_by_integer(T&, std::vector<uint8_t>&){};

    void put_value(std::vector<uint8_t>&, std::vector<uint8_t>&){};
};

class MockPersistedStore {
  public:
    using ReadTransaction = MockTransaction;
    using WriteTransaction = MockTransaction;
    static MockTransaction::Ptr create_write_transaction() { return std::make_unique<MockTransaction>(); }
    static MockTransaction::Ptr create_read_transaction() { return std::make_unique<MockTransaction>(); }
};

/**
 * @brief A very basic 2-d array for use as a backing store for merkle trees.
 * Can store up to 'indices' nodes per row and 'levels' rows.
 */
template <typename PersistedStore> class ArrayStore {

  public:
    using ReadTransaction = typename PersistedStore::ReadTransaction;
    using WriteTransaction = typename PersistedStore::WriteTransaction;
    using ReadTransactionPtr = std::unique_ptr<ReadTransaction>;
    ArrayStore(const std::string& name, uint32_t depth, index_t indices = 1024)
        : map(std::vector<std::vector<std::pair<bool, std::vector<uint8_t>>>>(
              depth + 1,
              std::vector<std::pair<bool, std::vector<uint8_t>>>(
                  indices, std::pair<bool, std::vector<uint8_t>>(false, std::vector<uint8_t>()))))
    {
        meta.depth = depth;
        meta.name = name;
        meta.size = 0;
    }
    ~ArrayStore() = default;

    ArrayStore() = delete;
    ArrayStore(ArrayStore const& other) = delete;
    ArrayStore(ArrayStore const&& other) = delete;
    ArrayStore& operator=(ArrayStore const& other) = delete;
    ArrayStore& operator=(ArrayStore const&& other) = delete;

    void put_node(uint32_t level, index_t index, const std::vector<uint8_t>& data)
    {
        map[level][index] = std::make_pair(true, data);
    }
    bool get_node(uint32_t level, index_t index, std::vector<uint8_t>& data, ReadTransaction&, bool) const
    {
        const std::pair<bool, std::vector<uint8_t>>& slot = map[level][index];
        if (slot.first) {
            data = slot.second;
        }
        return slot.first;
    }
    void put_meta(const index_t& size, const bb::fr& root)
    {
        meta.root = root;
        meta.size = size;
    }

    void get_meta(index_t& size, bb::fr& root, ReadTransaction&, bool) const
    {
        size = meta.size;
        root = meta.root;
    }

    void commit(){};
    void rollback(){};

    ReadTransactionPtr create_read_transactiono() { return std::make_unique<ReadTransaction>(); }

  private:
    std::vector<std::vector<std::pair<bool, std::vector<uint8_t>>>> map;
    TreeMeta meta;
};
} // namespace bb::crypto::merkle_tree