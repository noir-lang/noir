#pragma once

#include <condition_variable>
#include <lmdb.h>
#include <mutex>
#include <string>
namespace bb::crypto::merkle_tree {
/*
 * RAII wrapper around an LMDB environment.
 * Opens/creates the environemnt and manages read access to the enviroment.
 * The environment has an upper limit on the number of concurrent read transactions
 * and this is managed through the use of mutex/condition variables
 */
class LMDBEnvironment {
  public:
    /**
     * @brief Opens/creates the LMDB environment
     * @param directory The directory in which the environment is to be created
     * @param mapSizeKb The maximum size of the database, can be increased from a previously used value
     * @param maxNumDbs The maximum number of databases that can be created withn this environment
     * @param maxNumReaders The maximum number of concurrent read transactions permitted.
     */
    LMDBEnvironment(const std::string& directory, uint64_t mapSizeKb, uint32_t maxNumDBs, uint32_t maxNumReaders);
    LMDBEnvironment(const LMDBEnvironment& other) = delete;
    LMDBEnvironment(LMDBEnvironment&& other) = delete;
    LMDBEnvironment& operator=(const LMDBEnvironment& other) = delete;
    LMDBEnvironment& operator=(LMDBEnvironment&& other) = delete;

    ~LMDBEnvironment();

    MDB_env* underlying() const;

    void wait_for_reader();

    void release_reader();

  private:
    MDB_env* _mdbEnv;
    uint32_t _maxReaders;
    uint32_t _numReaders;
    std::mutex _readersLock;
    std::condition_variable _readersCondition;
};
} // namespace bb::crypto::merkle_tree