#pragma once
#include "hash_path.hpp"
#include "leveldb_store.hpp"
#include <leveldb/db.h>
#include <leveldb/write_batch.h>
#include <map>
#include <set>

namespace plonk {
namespace stdlib {
namespace merkle_tree {

using namespace leveldb;

class leveldb_tx {
  public:
    leveldb_tx(DB& db)
        : db_(db)
    {}

    Status Put(const Slice& key, const Slice& value)
    {
        puts_[key.ToString()] = value.ToString();
        deletes_.erase(key.ToString());
        return Status::OK();
    }

    Status Delete(const Slice& key)
    {
        puts_.erase(key.ToString());
        deletes_.insert(key.ToString());
        return Status::OK();
    };

    Status Get(const Slice& key, std::string* value)
    {
        if (deletes_.find(key.ToString()) != deletes_.end()) {
            return Status::NotFound("Not found.");
        }
        auto it = puts_.find(key.ToString());
        if (it != puts_.end()) {
            *value = it->second;
            return Status::OK();
        } else {
            return db_.Get(ReadOptions(), key, value);
        }
    }

    void populate_write_batch(leveldb::WriteBatch& batch) const
    {
        for (auto it : puts_) {
            batch.Put(it.first, it.second);
        }
        for (auto key : deletes_) {
            batch.Delete(key);
        }
    }

  private:
    std::map<std::string, std::string> puts_;
    std::set<std::string> deletes_;
    leveldb::DB& db_;
};

} // namespace merkle_tree
} // namespace stdlib
} // namespace plonk