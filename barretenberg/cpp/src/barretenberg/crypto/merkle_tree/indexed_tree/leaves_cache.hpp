#pragma once
#include "barretenberg/stdlib/primitives/field/field.hpp"
#include "indexed_leaf.hpp"

namespace bb::crypto::merkle_tree {

typedef uint256_t index_t;

/**
 * @brief Used to facilitate testing of the IndexedTree. Stores leaves in memory with an index for O(logN) retrieval of
 * 'low leaves'
 *
 */
class LeavesCache {
  public:
    index_t get_size() const;
    std::pair<bool, index_t> find_low_value(const bb::fr& new_value) const;
    indexed_leaf get_leaf(const index_t& index) const;
    void set_at_index(const index_t& index, const indexed_leaf& leaf, bool add_to_index);
    void append_leaf(const indexed_leaf& leaf);

  private:
    std::map<uint256_t, index_t> indices_;
    std::vector<indexed_leaf> leaves_;
};

} // namespace bb::crypto::merkle_tree