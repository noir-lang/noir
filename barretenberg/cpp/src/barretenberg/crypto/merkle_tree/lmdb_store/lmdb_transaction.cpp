#include "barretenberg/crypto/merkle_tree/lmdb_store/lmdb_transaction.hpp"
#include "barretenberg/crypto/merkle_tree/lmdb_store/callbacks.hpp"

namespace bb::crypto::merkle_tree {
LMDBTransaction::LMDBTransaction(LMDBEnvironment& env, bool readOnly)
    : _environment(env)
    , state(TransactionState::OPEN)
{
    MDB_txn* p = nullptr;
    call_lmdb_func(
        "mdb_txn_begin", mdb_txn_begin, _environment.underlying(), p, readOnly ? MDB_RDONLY : 0U, &_transaction);
}

LMDBTransaction::~LMDBTransaction() = default;

MDB_txn* LMDBTransaction::underlying() const
{
    return _transaction;
}

void LMDBTransaction::abort()
{
    if (state != TransactionState::OPEN) {
        return;
    }
    call_lmdb_func(mdb_txn_abort, _transaction);
    state = TransactionState::ABORTED;
}
} // namespace bb::crypto::merkle_tree