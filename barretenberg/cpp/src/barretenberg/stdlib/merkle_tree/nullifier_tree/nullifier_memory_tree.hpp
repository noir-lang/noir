#pragma once
#include "../hash.hpp"
#include "../memory_tree.hpp"
#include "nullifier_leaf.hpp"

namespace bb::plonk {
namespace stdlib {
namespace merkle_tree {

using namespace bb;

/**
 * An NullifierMemoryTree is structured just like a usual merkle tree:
 *
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
 *
 * 1. Initial state:
 *
 *                                        #
 *
 *                        #                               #
 *
 *                #               #               #               #
 *
 *            #       #       #       #        #       #       #       #
 *
 *  index     0       1       2       3        4       5       6       7
 *
 *  val       0       0       0       0        0       0       0       0
 *  nextIdx   0       0       0       0        0       0       0       0
 *  nextVal   0       0       0       0        0       0       0       0
 *
 * 2. Add new leaf with value 30
 *
 *  val       0       30      0       0        0       0       0       0
 *  nextIdx   1       0       0       0        0       0       0       0
 *  nextVal   30      0       0       0        0       0       0       0
 *
 * 3. Add new leaf with value 10
 *
 *  val       0       30      10      0        0       0       0       0
 *  nextIdx   2       0       1       0        0       0       0       0
 *  nextVal   10      0       30      0        0       0       0       0
 *
 * 4. Add new leaf with value 20
 *
 *  val       0       30      10      20       0       0       0       0
 *  nextIdx   2       0       3       1        0       0       0       0
 *  nextVal   10      0       20      30       0       0       0       0
 *
 * 5. Add new leaf with value 50
 *
 *  val       0       30      10      20       50      0       0       0
 *  nextIdx   2       4       3       1        0       0       0       0
 *  nextVal   10      50      20      30       0       0       0       0
 */
class NullifierMemoryTree : public MemoryTree {

  public:
    NullifierMemoryTree(size_t depth);

    using MemoryTree::get_hash_path;
    using MemoryTree::root;
    using MemoryTree::update_element;

    fr update_element(fr const& value);

    const std::vector<bb::fr>& get_hashes() { return hashes_; }
    const WrappedNullifierLeaf get_leaf(size_t index)
    {
        return (index < leaves_.size()) ? leaves_[index] : WrappedNullifierLeaf::zero();
    }
    const std::vector<WrappedNullifierLeaf>& get_leaves() { return leaves_; }

  protected:
    using MemoryTree::depth_;
    using MemoryTree::hashes_;
    using MemoryTree::root_;
    using MemoryTree::total_size_;
    std::vector<WrappedNullifierLeaf> leaves_;
};

} // namespace merkle_tree
} // namespace stdlib
} // namespace bb::plonk