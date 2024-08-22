#pragma once
#include "barretenberg/crypto/merkle_tree/lmdb_store/lmdb_transaction.hpp"

namespace bb::crypto::merkle_tree {

/*
 * RAII wrapper to construct a transaction for the purpose of creating/opening a database
 */
class LMDBDatabaseCreationTransaction : public LMDBTransaction {
  public:
    using Ptr = std::unique_ptr<LMDBDatabaseCreationTransaction>;

    LMDBDatabaseCreationTransaction(LMDBEnvironment& env);
    LMDBDatabaseCreationTransaction(const LMDBDatabaseCreationTransaction& other) = delete;
    LMDBDatabaseCreationTransaction(LMDBDatabaseCreationTransaction&& other) = delete;
    LMDBDatabaseCreationTransaction& operator=(const LMDBDatabaseCreationTransaction& other) = delete;
    LMDBDatabaseCreationTransaction& operator=(LMDBDatabaseCreationTransaction&& other) = delete;

    ~LMDBDatabaseCreationTransaction() override = default;
    void commit() const;
};

} // namespace bb::crypto::merkle_tree