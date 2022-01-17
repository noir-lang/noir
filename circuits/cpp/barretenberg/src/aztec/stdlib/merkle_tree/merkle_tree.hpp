#pragma once
#include "hash_path.hpp"
#include <stdlib/primitives/field/field.hpp>

namespace plonk {
namespace stdlib {
namespace merkle_tree {

using namespace barretenberg;

class LevelDbStore;
class MemoryStore;

template <typename Store> class MerkleTree {
  public:
    typedef uint256_t index_t;

    MerkleTree(Store& store, size_t depth, uint8_t tree_id = 0);
    MerkleTree(MerkleTree const& other) = delete;
    MerkleTree(MerkleTree&& other);
    ~MerkleTree();

    fr_hash_path get_hash_path(index_t index);

    fr update_element(index_t index, fr const& value);

    fr root() const;

    size_t depth() const { return depth_; }

    index_t size() const;

  private:
    void load_metadata();

    /**
     * Computes the root hash of a tree of `height`, that is empty other than `value` at `index`.
     *
     * @param height: The tree depth
     * @param index: the index of the non-empty leaf
     * @param value: the value to be stored in the non-empty leaf
     *
     * @see Check full documentation: https://hackmd.io/2zyJc6QhRuugyH8D78Tbqg?view
     */
    fr update_element(fr const& root, fr const& value, index_t index, size_t height);

    fr get_element(fr const& root, index_t index, size_t height);

    /**
     * Computes the root hash of a tree of `height`, that is empty other than `value` at `index`.
     *
     * @param height: The tree depth
     * @param index: the index of the non-empty leaf
     * @param value: the value to be stored in the non-empty leaf
     */
    fr compute_zero_path_hash(size_t height, index_t index, fr const& value);

    /**
     * Given child nodes `a` and `b` and index of `a`, compute their parent node `p` and store [p : (a, b)].
     *
     * @param a_index: the index of the child node `a`
     * @param a: child node
     * @param b: child node
     * @param height: the height of the parent node
     */
    fr binary_put(index_t a_index, fr const& a, fr const& b, size_t height);

    fr fork_stump(
        fr const& value1, index_t index1, fr const& value2, index_t index2, size_t height, size_t stump_height);

    /**
     * Stores a parent node and child nodes in the database as [key : (left, right)].
     *
     * @param key: The node value to be stored as key
     * @param left: the left child node
     * @param right: the right child node
     */
    void put(fr const& key, fr const& left, fr const& right);

    /**
     * Stores a stump [key : (value, index, true)] in the memory.
     * The additional byte `true` is to denote this is a stump.
     *
     * @param key: The node value to be stored as key
     * @param value: value of the non-empty leaf in the stump
     * @param index: the index of the non-empty leaf in the stump
     */
    void put_stump(fr const& key, index_t index, fr const& value);

    void remove(fr const& key);

  private:
    Store& store_;
    std::vector<fr> zero_hashes_;
    size_t depth_;
    uint8_t tree_id_;
};

extern template class MerkleTree<LevelDbStore>;
extern template class MerkleTree<MemoryStore>;

typedef MerkleTree<LevelDbStore> LevelDbTree;

} // namespace merkle_tree
} // namespace stdlib
} // namespace plonk