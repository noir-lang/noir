#include "lmdb_store.hpp"
#include "barretenberg/crypto/merkle_tree/types.hpp"
#include "barretenberg/numeric/uint128/uint128.hpp"
#include "barretenberg/numeric/uint256/uint256.hpp"
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <lmdb.h>
#include <vector>

namespace bb::crypto::merkle_tree {
LMDBStore::LMDBStore(
    LMDBEnvironment& environment, std::string name, bool integerKeys, bool reverseKeys, MDB_cmp_func* cmp)
    : _environment(environment)
    , _name(std::move(name))
    , _database(_environment, LMDBDatabaseCreationTransaction(_environment), _name, integerKeys, reverseKeys, cmp)
{}

LMDBWriteTransaction::Ptr LMDBStore::create_write_transaction() const
{
    return std::make_unique<LMDBWriteTransaction>(_environment, _database);
}
LMDBReadTransaction::Ptr LMDBStore::create_read_transaction()
{
    _environment.wait_for_reader();
    return std::make_unique<LMDBReadTransaction>(_environment, _database);
}
} // namespace bb::crypto::merkle_tree