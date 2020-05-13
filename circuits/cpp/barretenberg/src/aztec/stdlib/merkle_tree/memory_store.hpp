#pragma once
#include "hash_path.hpp"
#include <map>
#include <set>
#include <common/streams.hpp>

namespace plonk {
namespace stdlib {
namespace merkle_tree {

class MemoryStore {
  public:
    MemoryStore() {}

    MemoryStore(MemoryStore const& rhs) = default;
    MemoryStore(MemoryStore&& rhs) = default;
    MemoryStore& operator=(MemoryStore const& rhs) = default;
    MemoryStore& operator=(MemoryStore&& rhs) = default;

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

    bool get(std::vector<uint8_t> const& key, std::vector<uint8_t>& value) { return get(to_string(key), value); }

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
            auto it = store_.find(key);
            if (it != store_.end()) {
                value = { it->second.begin(), it->second.end() };
                return true;
            }
            return false;
        }
    }

    void commit()
    {
        for (auto it : puts_) {
            store_.insert(it);
        }
        for (auto key : deletes_) {
            store_.erase(key);
        }
        puts_.clear();
        deletes_.clear();
    }

    void rollback()
    {
        puts_.clear();
        deletes_.clear();
    }

  private:
    std::string to_string(std::vector<uint8_t> const& input) { return std::string((char*)input.data(), input.size()); }

    std::map<std::string, std::string> store_;
    std::map<std::string, std::string> puts_;
    std::set<std::string> deletes_;
};

} // namespace merkle_tree
} // namespace stdlib
} // namespace plonk
