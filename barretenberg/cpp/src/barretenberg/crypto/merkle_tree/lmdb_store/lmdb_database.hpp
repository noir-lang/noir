#pragma once
#include "barretenberg/crypto/merkle_tree/lmdb_store/callbacks.hpp"
#include "barretenberg/crypto/merkle_tree/lmdb_store/lmdb_db_transaction.hpp"
#include "barretenberg/crypto/merkle_tree/lmdb_store/lmdb_environment.hpp"

namespace bb::crypto::merkle_tree {
/**
 * RAII wrapper atound the opening and closing of an LMDB database
 * Contains a reference to its LMDB environment
 */
class LMDBDatabase {
  public:
    LMDBDatabase(const LMDBEnvironment& env,
                 const LMDBDatabaseCreationTransaction& transaction,
                 const std::string& name,
                 bool integerKeys = false,
                 bool reverseKeys = false,
                 MDB_cmp_func* cmp = nullptr);

    LMDBDatabase(const LMDBDatabase& other) = delete;
    LMDBDatabase(LMDBDatabase&& other) = delete;
    LMDBDatabase& operator=(const LMDBDatabase& other) = delete;
    LMDBDatabase& operator=(LMDBDatabase&& other) = delete;

    ~LMDBDatabase();

    const MDB_dbi& underlying() const;

  private:
    MDB_dbi _dbi;
    const LMDBEnvironment& _environment;
};
} // namespace bb::crypto::merkle_tree
