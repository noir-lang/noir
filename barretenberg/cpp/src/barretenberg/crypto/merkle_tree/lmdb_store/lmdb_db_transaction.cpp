#include "barretenberg/crypto/merkle_tree/lmdb_store/lmdb_db_transaction.hpp"
#include "barretenberg/crypto/merkle_tree/lmdb_store/callbacks.hpp"

namespace bb::crypto::merkle_tree {
LMDBDatabaseCreationTransaction::LMDBDatabaseCreationTransaction(LMDBEnvironment& env)
    : LMDBTransaction(env)
{}
void LMDBDatabaseCreationTransaction::commit() const
{
    call_lmdb_func("mdb_txn_commit", mdb_txn_commit, _transaction);
}
} // namespace bb::crypto::merkle_tree