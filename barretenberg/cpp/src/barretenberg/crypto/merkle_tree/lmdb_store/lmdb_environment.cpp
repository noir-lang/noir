#include "barretenberg/crypto/merkle_tree/lmdb_store/lmdb_environment.hpp"
#include "barretenberg/crypto/merkle_tree/lmdb_store/callbacks.hpp"
#include <stdexcept>
#include <sys/stat.h>

namespace bb::crypto::merkle_tree {
LMDBEnvironment::LMDBEnvironment(const std::string& directory,
                                 uint64_t mapSizeKB,
                                 uint32_t maxNumDBs,
                                 uint32_t maxNumReaders)
    : _maxReaders(maxNumReaders)
    , _numReaders(0)
{
    call_lmdb_func("mdb_env_create", mdb_env_create, &_mdbEnv);
    uint64_t kb = 1024;
    uint64_t totalMapSize = kb * mapSizeKB;
    uint32_t flags = MDB_NOTLS;
    try {
        call_lmdb_func("mdb_env_set_mapsize", mdb_env_set_mapsize, _mdbEnv, totalMapSize);
        call_lmdb_func("mdb_env_set_maxdbs", mdb_env_set_maxdbs, _mdbEnv, static_cast<MDB_dbi>(maxNumDBs));
        call_lmdb_func("mdb_env_set_maxreaders", mdb_env_set_maxreaders, _mdbEnv, maxNumReaders);
        call_lmdb_func("mdb_env_open",
                       mdb_env_open,
                       _mdbEnv,
                       directory.c_str(),
                       flags,
                       static_cast<mdb_mode_t>(S_IRWXU | S_IRWXG | S_IRWXO));
    } catch (std::runtime_error& error) {
        call_lmdb_func(mdb_env_close, _mdbEnv);
        throw error;
    }
}

void LMDBEnvironment::wait_for_reader()
{
    std::unique_lock lock(_readersLock);
    if (_numReaders >= _maxReaders) {
        _readersCondition.wait(lock, [&] { return _numReaders < _maxReaders; });
    }
    ++_numReaders;
}

void LMDBEnvironment::release_reader()
{
    std::unique_lock lock(_readersLock);
    --_numReaders;
    _readersCondition.notify_one();
}

LMDBEnvironment::~LMDBEnvironment()
{
    call_lmdb_func(mdb_env_close, _mdbEnv);
}

MDB_env* LMDBEnvironment::underlying() const
{
    return _mdbEnv;
}
} // namespace bb::crypto::merkle_tree