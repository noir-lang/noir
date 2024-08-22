#pragma once
#include "barretenberg/common/serialize.hpp"
#include "barretenberg/crypto/merkle_tree/lmdb_store/callbacks.hpp"
#include "barretenberg/crypto/merkle_tree/lmdb_store/lmdb_database.hpp"
#include "barretenberg/crypto/merkle_tree/lmdb_store/lmdb_transaction.hpp"
#include "barretenberg/crypto/merkle_tree/types.hpp"

namespace bb::crypto::merkle_tree {

/**
 * RAII wrapper for an LMDB write transaction.
 * Provides methods for writing values by their key.
 * Must be either committed to persist the changes or aborted to roll them back.
 * Will automatically abort the transaction during destruction if changes have not been committed.
 */

class LMDBWriteTransaction : public LMDBTransaction {
  public:
    using Ptr = std::unique_ptr<LMDBWriteTransaction>;

    LMDBWriteTransaction(LMDBEnvironment& env, const LMDBDatabase& database);
    LMDBWriteTransaction(const LMDBWriteTransaction& other) = delete;
    LMDBWriteTransaction(LMDBWriteTransaction&& other) = delete;
    LMDBWriteTransaction& operator=(const LMDBWriteTransaction& other) = delete;
    LMDBWriteTransaction& operator=(LMDBWriteTransaction&& other) = delete;
    ~LMDBWriteTransaction() override;

    void put_node(uint32_t level, index_t index, std::vector<uint8_t>& data);

    template <typename T> void put_value(T& key, std::vector<uint8_t>& data);

    void put_value(std::vector<uint8_t>& key, std::vector<uint8_t>& data);

    void commit();

    void try_abort();

  protected:
    const LMDBDatabase& _database;
};

template <typename T> void LMDBWriteTransaction::put_value(T& key, std::vector<uint8_t>& data)
{
    std::vector<uint8_t> keyBuffer = serialise_key(key);
    put_value(keyBuffer, data);
}
} // namespace bb::crypto::merkle_tree