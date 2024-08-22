#pragma once
#include "barretenberg/crypto/merkle_tree/lmdb_store/lmdb_database.hpp"
#include "barretenberg/crypto/merkle_tree/lmdb_store/lmdb_read_transaction.hpp"
#include "barretenberg/crypto/merkle_tree/lmdb_store/lmdb_write_transaction.hpp"

namespace bb::crypto::merkle_tree {

/**
 * Creates an named LMDB 'Store' abstraction on top of an environment.
 * Provides methods for creating read and write transactions against the store.
 */

class LMDBStore {

  public:
    using ReadTransaction = LMDBReadTransaction;
    using WriteTransaction = LMDBWriteTransaction;
    LMDBStore(LMDBEnvironment& environment,
              std::string name,
              bool integerKeys = false,
              bool reverseKeys = false,
              MDB_cmp_func* cmp = nullptr);
    LMDBStore(const LMDBStore& other) = delete;
    LMDBStore(LMDBStore&& other) = delete;
    LMDBStore& operator=(const LMDBStore& other) = delete;
    LMDBStore& operator=(LMDBStore&& other) = delete;
    ~LMDBStore() = default;

    LMDBWriteTransaction::Ptr create_write_transaction() const;
    LMDBReadTransaction::Ptr create_read_transaction();

  private:
    LMDBEnvironment& _environment;
    const std::string _name;
    LMDBDatabase _database;
};
} // namespace bb::crypto::merkle_tree