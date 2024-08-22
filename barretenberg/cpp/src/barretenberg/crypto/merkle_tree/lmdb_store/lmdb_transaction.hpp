#pragma once
#include "barretenberg/crypto/merkle_tree/lmdb_store/lmdb_environment.hpp"

namespace bb::crypto::merkle_tree {

/*
 * Abstract base class to represent and LMDB transaction.
 * Needs to be sub-classed to be either a read or write transaction.
 */

enum TransactionState {
    OPEN,
    COMMITTED,
    ABORTED,
};

class LMDBTransaction {
  public:
    LMDBTransaction(LMDBEnvironment& env, bool readOnly = false);
    LMDBTransaction(const LMDBTransaction& other) = delete;
    LMDBTransaction(LMDBTransaction&& other) = delete;
    LMDBTransaction& operator=(const LMDBTransaction& other) = delete;
    LMDBTransaction& operator=(LMDBTransaction&& other) = delete;

    virtual ~LMDBTransaction() = 0;

    MDB_txn* underlying() const;

    /*
     * Rolls back the transaction.
     * Must be called by read transactions to signal the end of the transaction.
     * Must be called by write transactions if the changes are not to be committed.
     */
    virtual void abort();

  protected:
    LMDBEnvironment& _environment;
    MDB_txn* _transaction;
    TransactionState state;
};
} // namespace bb::crypto::merkle_tree