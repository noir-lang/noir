#pragma once
#include "hash_path.hpp"

namespace plonk {
namespace stdlib {
namespace merkle_tree {

using namespace barretenberg;

/**
 * A MemoryTree is structured as follows:
 *                                       hashes_
 *    +------------------------------------------------------------------------------+
 *    |  0 -> h_{0,0}  h_{0,1}  h_{0,2}  h_{0,3}  h_{0,4}  h_{0,5}  h_{0,6}  h_{0,7} |
 *  i |                                                                              |
 *  n |  8 -> h_{1,0}  h_{1,1}  h_{1,2}  h_{1,3}                                     |
 *  d |                                                                              |
 *  e | 12 -> h_{2,0}  h_{2,1}                                                       |
 *  x |                                                                              |
 *    | 14 -> h_{3,0}                                                                |
 *    +------------------------------------------------------------------------------+
 *
 * Here, depth_ = 3 and {h_{0,j}}_{i=0..7} are leaf values.
 * Also, root_ = h_{3,0} and total_size_ = (2 * 8 - 2) = 14.
 * Lastly, h_{i,j} = hash( h_{i-1,2j}, h_{i-1,2j+1} ) where i > 1.
 */
class MemoryTree {
  public:
    MemoryTree(size_t depth);

    fr_hash_path get_hash_path(size_t index);

    fr update_element(size_t index, fr const& value);

    fr root() const { return root_; }

  private:
    size_t depth_;
    size_t total_size_;
    barretenberg::fr root_;
    std::vector<barretenberg::fr> hashes_;
};

} // namespace merkle_tree
} // namespace stdlib
} // namespace plonk