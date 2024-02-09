#pragma once
#include "../hash.hpp"
#include "../merkle_tree.hpp"
#include "nullifier_leaf.hpp"

namespace bb::crypto::merkle_tree {

template <typename Store, typename HashingPolicy> class NullifierTree : public MerkleTree<Store, HashingPolicy> {
  public:
    typedef uint256_t index_t;

    NullifierTree(Store& store, size_t depth, size_t initial_size = 1, uint8_t tree_id = 0);
    NullifierTree(NullifierTree const& other) = delete;
    NullifierTree(NullifierTree&& other);
    ~NullifierTree();

    using MerkleTree<Store, HashingPolicy>::get_hash_path;
    using MerkleTree<Store, HashingPolicy>::root;
    using MerkleTree<Store, HashingPolicy>::size;
    using MerkleTree<Store, HashingPolicy>::depth;

    fr update_element(fr const& value);

  private:
    using MerkleTree<Store, HashingPolicy>::update_element;
    using MerkleTree<Store, HashingPolicy>::get_element;
    using MerkleTree<Store, HashingPolicy>::compute_zero_path_hash;

  private:
    using MerkleTree<Store, HashingPolicy>::store_;
    using MerkleTree<Store, HashingPolicy>::zero_hashes_;
    using MerkleTree<Store, HashingPolicy>::depth_;
    using MerkleTree<Store, HashingPolicy>::tree_id_;
    std::vector<WrappedNullifierLeaf<HashingPolicy>> leaves;
};

} // namespace bb::crypto::merkle_tree
