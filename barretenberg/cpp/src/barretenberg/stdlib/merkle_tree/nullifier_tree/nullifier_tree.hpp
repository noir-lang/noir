#pragma once
#include "../hash.hpp"
#include "../merkle_tree.hpp"
#include "nullifier_leaf.hpp"

namespace bb::plonk {
namespace stdlib {
namespace merkle_tree {

using namespace bb;

template <typename Store> class NullifierTree : public MerkleTree<Store> {
  public:
    typedef uint256_t index_t;

    NullifierTree(Store& store, size_t depth, uint8_t tree_id = 0);
    NullifierTree(NullifierTree const& other) = delete;
    NullifierTree(NullifierTree&& other);
    ~NullifierTree();

    using MerkleTree<Store>::get_hash_path;
    using MerkleTree<Store>::root;
    using MerkleTree<Store>::size;
    using MerkleTree<Store>::depth;

    fr update_element(fr const& value);

  private:
    using MerkleTree<Store>::update_element;
    using MerkleTree<Store>::get_element;
    using MerkleTree<Store>::compute_zero_path_hash;

  private:
    using MerkleTree<Store>::store_;
    using MerkleTree<Store>::zero_hashes_;
    using MerkleTree<Store>::depth_;
    using MerkleTree<Store>::tree_id_;
    std::vector<WrappedNullifierLeaf> leaves;
};

} // namespace merkle_tree
} // namespace stdlib
} // namespace bb::plonk
