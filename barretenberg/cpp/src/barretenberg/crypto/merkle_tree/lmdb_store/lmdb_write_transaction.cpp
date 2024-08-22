

#include "barretenberg/crypto/merkle_tree/lmdb_store/lmdb_write_transaction.hpp"
#include "barretenberg/crypto/merkle_tree/lmdb_store/callbacks.hpp"

namespace bb::crypto::merkle_tree {

LMDBWriteTransaction::LMDBWriteTransaction(LMDBEnvironment& env, const LMDBDatabase& database)
    : LMDBTransaction(env)
    , _database(database)
{}

LMDBWriteTransaction::~LMDBWriteTransaction()
{
    try_abort();
}

void LMDBWriteTransaction::commit()
{
    if (state == TransactionState::ABORTED) {
        throw std::runtime_error("Tried to commit reverted transaction");
    }
    call_lmdb_func("mdb_txn_commit", mdb_txn_commit, _transaction);
    state = TransactionState::COMMITTED;
}

void LMDBWriteTransaction::try_abort()
{
    if (state != TransactionState::OPEN) {
        return;
    }
    LMDBTransaction::abort();
}

void LMDBWriteTransaction::put_node(uint32_t level, index_t index, std::vector<uint8_t>& data)
{
    NodeKeyType key = get_key_for_node(level, index);
    put_value(key, data);
}

void LMDBWriteTransaction::put_value(std::vector<uint8_t>& key, std::vector<uint8_t>& data)
{
    MDB_val dbKey;
    dbKey.mv_size = key.size();
    dbKey.mv_data = (void*)key.data();

    MDB_val dbVal;
    dbVal.mv_size = data.size();
    dbVal.mv_data = (void*)data.data();
    call_lmdb_func("mdb_put", mdb_put, underlying(), _database.underlying(), &dbKey, &dbVal, 0U);
}
} // namespace bb::crypto::merkle_tree
