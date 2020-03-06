#pragma once
#include "../field/field.hpp"
#include "hash_path.hpp"
#include "leveldb_tx.hpp"
#include <leveldb/db.h>
#include <leveldb/write_batch.h>

namespace plonk {
namespace stdlib {
namespace merkle_tree {

using namespace barretenberg;

class LevelDbStore {
  public:
    typedef uint128_t index_t;
    typedef std::string value_t;

    LevelDbStore(std::string const& path, size_t depth);
    LevelDbStore(LevelDbStore const& other) = delete;
    LevelDbStore(LevelDbStore&& other) = default;

    fr_hash_path get_hash_path(index_t index);

    void update_element(index_t index, value_t const& value);

    value_t get_element(index_t index);

    fr root() const;

    size_t depth() const { return depth_; }

    size_t size() const;

    void commit();

    void rollback();

  private:
    fr update_element(fr const& root, fr const& value, index_t index, size_t height);

    fr get_element(fr const& root, index_t index, size_t height);

    fr compute_zero_path_hash(size_t height, index_t index, fr const& value);

    fr binary_put(index_t a_index, fr const& a, fr const& b, size_t height);

    fr fork_stump(fr const& value1,
                           index_t index1,
                           fr const& value2,
                           index_t index2,
                           size_t height,
                           size_t stump_height);

    void put(fr const& key, fr const& left, fr const& right);

    void put_stump(fr const& key, index_t index, fr const& value);

  private:
    static constexpr size_t LEAF_BYTES = 64;
    std::unique_ptr<leveldb::DB> db_;
    std::unique_ptr<leveldb_tx> tx_;
    std::vector<fr> zero_hashes_;
    size_t depth_;
    size_t size_;
    fr root_;
};

} // namespace merkle_tree
} // namespace stdlib
} // namespace plonk