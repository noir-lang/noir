#include "barretenberg/crypto/merkle_tree/lmdb_store/lmdb_read_transaction.hpp"
#include "barretenberg/crypto/merkle_tree/lmdb_store/callbacks.hpp"
#include <cstdint>

namespace bb::crypto::merkle_tree {
LMDBReadTransaction::LMDBReadTransaction(LMDBEnvironment& env, const LMDBDatabase& database)
    : LMDBTransaction(env, true)
    , _database(database)
{}

LMDBReadTransaction::~LMDBReadTransaction()
{
    abort();
}

void LMDBReadTransaction::abort()
{
    LMDBTransaction::abort();
    _environment.release_reader();
}

bool LMDBReadTransaction::get_value(std::vector<uint8_t>& key, std::vector<uint8_t>& data) const
{
    MDB_val dbKey;
    dbKey.mv_size = key.size();
    dbKey.mv_data = (void*)key.data();

    MDB_val dbVal;
    if (!call_lmdb_func(mdb_get, underlying(), _database.underlying(), &dbKey, &dbVal)) {
        return false;
    }
    copy_to_vector(dbVal, data);
    return true;
}

bool LMDBReadTransaction::get_node(uint32_t level, index_t index, std::vector<uint8_t>& data) const
{
    NodeKeyType key = get_key_for_node(level, index);
    return get_value(key, data);
}
} // namespace bb::crypto::merkle_tree
