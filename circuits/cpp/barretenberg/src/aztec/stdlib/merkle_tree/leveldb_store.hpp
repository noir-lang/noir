#pragma once
#include "hash_path.hpp"
#include <leveldb/db.h>
#include <leveldb/write_batch.h>
#include <map>
#include <set>
#include <common/streams.hpp>

namespace plonk {
namespace stdlib {
namespace merkle_tree {

using namespace leveldb;

inline std::string to_string(std::vector<uint8_t> const& input) {
    return std::string((char*)input.data(), input.size());
}

class LevelDbStore {
  public:
    LevelDbStore(std::string const& db_path) {
        leveldb::DB* db;
        leveldb::Options options;
        options.create_if_missing = true;
        options.compression = leveldb::kNoCompression;
        leveldb::Status status = leveldb::DB::Open(options, db_path, &db);
        ASSERT(status.ok());
        db_.reset(db);
    }

    static void destroy(std::string path)
    {
        leveldb::DestroyDB(path, leveldb::Options());
    }

    bool put(std::vector<uint8_t> const& key, std::vector<uint8_t> const& value)
    {
        auto key_str = to_string(key);
        return put(key_str, value);
    }

    bool put(std::string const& key, std::vector<uint8_t> const& value)
    {
        puts_[key] = to_string(value);
        deletes_.erase(key);
        return true;
    }

    bool del(std::vector<uint8_t> const& key)
    {
        auto key_str = to_string(key);
        puts_.erase(key_str);
        deletes_.insert(key_str);
        return true;
    };

    bool get(std::vector<uint8_t> const& key, std::vector<uint8_t>& value) {
        return get(to_string(key), value);
    }

    bool get(std::string const& key, std::vector<uint8_t>& value)
    {
        if (deletes_.find(key) != deletes_.end()) {
            return false;
        }
        auto it = puts_.find(key);
        if (it != puts_.end()) {
            value = std::vector<uint8_t>(it->second.begin(), it->second.end());
            return true;
        } else {
            std::string result;
            leveldb::Status status = db_->Get(ReadOptions(), key, &result);
            value = { result.begin(), result.end() };
            return status.ok();
        }
    }

    void commit()
    {
        leveldb::WriteBatch batch;
        for (auto it : puts_) {
            batch.Put(it.first, it.second);
        }
        for (auto key : deletes_) {
            batch.Delete(key);
        }
        db_->Write(leveldb::WriteOptions(), &batch);
        puts_.clear();
        deletes_.clear();
    }

    void rollback() {
        puts_.clear();
        deletes_.clear();
    }

  private:
    std::unique_ptr<leveldb::DB> db_;
    std::map<std::string, std::string> puts_;
    std::set<std::string> deletes_;
};

} // namespace merkle_tree
} // namespace stdlib
} // namespace plonk