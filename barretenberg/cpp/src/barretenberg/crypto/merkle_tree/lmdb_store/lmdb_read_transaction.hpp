#pragma once
#include "barretenberg/common/serialize.hpp"
#include "barretenberg/crypto/merkle_tree/lmdb_store/callbacks.hpp"
#include "barretenberg/crypto/merkle_tree/lmdb_store/lmdb_database.hpp"
#include "barretenberg/crypto/merkle_tree/lmdb_store/lmdb_transaction.hpp"
#include "barretenberg/crypto/merkle_tree/types.hpp"
#include <cstdint>
#include <cstring>
#include <vector>

namespace bb::crypto::merkle_tree {

/**
 * RAII wrapper around a read transaction.
 * Contains various methods for retrieving values by their keys.
 * Aborts the transaction upon object destruction.
 */
class LMDBReadTransaction : public LMDBTransaction {
  public:
    using Ptr = std::unique_ptr<LMDBReadTransaction>;

    LMDBReadTransaction(LMDBEnvironment& env, const LMDBDatabase& database);
    LMDBReadTransaction(const LMDBReadTransaction& other) = delete;
    LMDBReadTransaction(LMDBReadTransaction&& other) = delete;
    LMDBReadTransaction& operator=(const LMDBReadTransaction& other) = delete;
    LMDBReadTransaction& operator=(LMDBReadTransaction&& other) = delete;

    ~LMDBReadTransaction() override;

    template <typename T> bool get_value_or_previous(T& key, std::vector<uint8_t>& data) const;

    bool get_node(uint32_t level, index_t index, std::vector<uint8_t>& data) const;

    template <typename T> bool get_value(T& key, std::vector<uint8_t>& data) const;

    bool get_value(std::vector<uint8_t>& key, std::vector<uint8_t>& data) const;

    void abort() override;

  protected:
    const LMDBDatabase& _database;
};

template <typename T> bool LMDBReadTransaction::get_value(T& key, std::vector<uint8_t>& data) const
{
    std::vector<uint8_t> keyBuffer = serialise_key(key);
    return get_value(keyBuffer, data);
}

template <typename T> bool LMDBReadTransaction::get_value_or_previous(T& key, std::vector<uint8_t>& data) const
{
    std::vector<uint8_t> keyBuffer = serialise_key(key);
    uint32_t keySize = static_cast<uint32_t>(keyBuffer.size());
    MDB_cursor* cursor = nullptr;
    call_lmdb_func("mdb_cursor_open", mdb_cursor_open, underlying(), _database.underlying(), &cursor);

    MDB_val dbKey;
    dbKey.mv_size = keySize;
    dbKey.mv_data = (void*)keyBuffer.data();

    MDB_val dbVal;

    bool success = false;

    // Look for the key >= to that provided
    int code = mdb_cursor_get(cursor, &dbKey, &dbVal, MDB_SET_RANGE);
    if (code == 0) {
        // we found the key, now determine if it is the exact key
        std::vector<uint8_t> temp = mdb_val_to_vector(dbKey);
        if (keyBuffer == temp) {
            // we have the exact key
            copy_to_vector(dbVal, data);
            success = true;
        } else {
            // We have a key of the same size but larger value OR a larger size
            // either way we now need to find the previous key
            code = mdb_cursor_get(cursor, &dbKey, &dbVal, MDB_PREV);
            if (code == 0) {
                // We have found a previous key. It could be of the same size but smaller value, or smaller size which
                // is equal to not found
                if (dbKey.mv_size != keySize) {
                    // There is no previous key, do nothing
                } else {
                    copy_to_vector(dbVal, data);
                    deserialise_key(dbKey.mv_data, key);
                    success = true;
                }
            } else if (code == MDB_NOTFOUND) {
                // There is no previous key, do nothing
            } else {
                throw_error("get_value_or_previous::mdb_cursor_get", code);
            }
        }
    } else if (code == MDB_NOTFOUND) {
        // The key was not found, use the last key in the db
        code = mdb_cursor_get(cursor, &dbKey, &dbVal, MDB_PREV);
        if (code == 0) {
            // We found the last key, but we need to ensure it is the same size
            if (dbKey.mv_size != keySize) {
                // The key is not the same size, same as not found, do nothing
            } else {
                copy_to_vector(dbVal, data);
                deserialise_key(dbKey.mv_data, key);
                success = true;
            }
        } else if (code == MDB_NOTFOUND) {
            // DB is empty?
        } else {
            throw_error("get_value_or_previous::mdb_cursor_get", code);
        }
    } else {
        throw_error("get_value_or_previous::mdb_cursor_get", code);
    }
    call_lmdb_func(mdb_cursor_close, cursor);
    return success;
}
} // namespace bb::crypto::merkle_tree