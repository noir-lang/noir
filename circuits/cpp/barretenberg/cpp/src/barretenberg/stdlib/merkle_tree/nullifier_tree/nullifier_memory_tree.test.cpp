#include "nullifier_memory_tree.hpp"
#include <gtest/gtest.h>

using namespace barretenberg;
using namespace proof_system::plonk::stdlib::merkle_tree;

void print_tree(const size_t depth, std::vector<fr> hashes, std::string const& msg)
{
    info("\n", msg);
    size_t offset = 0;
    for (size_t i = 0; i < depth; i++) {
        info("i = ", i);
        size_t layer_size = (1UL << (depth - i));
        for (size_t j = 0; j < layer_size; j++) {
            info("j = ", j, ": ", hashes[offset + j]);
        }
        offset += layer_size;
    }
}

bool check_hash_path(const fr& root, const fr_hash_path& path, const nullifier_leaf& leaf_value, const size_t idx)
{
    auto current = leaf_value.hash();
    size_t depth_ = path.size();
    size_t index = idx;
    for (size_t i = 0; i < depth_; ++i) {
        fr left = (index & 1) ? path[i].first : current;
        fr right = (index & 1) ? current : path[i].second;
        current = hash_pair_native(left, right);
        index >>= 1;
    }
    return current == root;
}

TEST(crypto_nullifier_tree, test_nullifier_memory)
{
    // Create a depth-3 indexed merkle tree
    constexpr size_t depth = 3;
    NullifierMemoryTree tree(depth);

    /**
     * Intial state:
     *
     *  index     0       1       2       3        4       5       6       7
     *  ---------------------------------------------------------------------
     *  val       0       0       0       0        0       0       0       0
     *  nextIdx   0       0       0       0        0       0       0       0
     *  nextVal   0       0       0       0        0       0       0       0
     */
    nullifier_leaf zero_leaf = { 0, 0, 0 };
    EXPECT_EQ(tree.get_leaves().size(), 1);
    EXPECT_EQ(tree.get_leaves()[0], zero_leaf);

    /**
     * Add new value 30:
     *
     *  index     0       1       2       3        4       5       6       7
     *  ---------------------------------------------------------------------
     *  val       0       30      0       0        0       0       0       0
     *  nextIdx   1       0       0       0        0       0       0       0
     *  nextVal   30      0       0       0        0       0       0       0
     */
    tree.update_element(30);
    EXPECT_EQ(tree.get_leaves().size(), 2);
    EXPECT_EQ(tree.get_leaves()[0].hash(), WrappedNullifierLeaf({ 0, 1, 30 }).hash());
    EXPECT_EQ(tree.get_leaves()[1].hash(), WrappedNullifierLeaf({ 30, 0, 0 }).hash());

    /**
     * Add new value 10:
     *
     *  index     0       1       2       3        4       5       6       7
     *  ---------------------------------------------------------------------
     *  val       0       30      10      0        0       0       0       0
     *  nextIdx   2       0       1       0        0       0       0       0
     *  nextVal   10      0       30      0        0       0       0       0
     */
    tree.update_element(10);
    EXPECT_EQ(tree.get_leaves().size(), 3);
    EXPECT_EQ(tree.get_leaves()[0].hash(), WrappedNullifierLeaf({ 0, 2, 10 }).hash());
    EXPECT_EQ(tree.get_leaves()[1].hash(), WrappedNullifierLeaf({ 30, 0, 0 }).hash());
    EXPECT_EQ(tree.get_leaves()[2].hash(), WrappedNullifierLeaf({ 10, 1, 30 }).hash());

    /**
     * Add new value 20:
     *
     *  index     0       1       2       3        4       5       6       7
     *  ---------------------------------------------------------------------
     *  val       0       30      10      20       0       0       0       0
     *  nextIdx   2       0       3       1        0       0       0       0
     *  nextVal   10      0       20      30       0       0       0       0
     */
    tree.update_element(20);
    EXPECT_EQ(tree.get_leaves().size(), 4);
    EXPECT_EQ(tree.get_leaves()[0].hash(), WrappedNullifierLeaf({ 0, 2, 10 }).hash());
    EXPECT_EQ(tree.get_leaves()[1].hash(), WrappedNullifierLeaf({ 30, 0, 0 }).hash());
    EXPECT_EQ(tree.get_leaves()[2].hash(), WrappedNullifierLeaf({ 10, 3, 20 }).hash());
    EXPECT_EQ(tree.get_leaves()[3].hash(), WrappedNullifierLeaf({ 20, 1, 30 }).hash());

    // Adding the same value must not affect anything
    tree.update_element(20);
    EXPECT_EQ(tree.get_leaves().size(), 4);
    EXPECT_EQ(tree.get_leaves()[0].hash(), WrappedNullifierLeaf({ 0, 2, 10 }).hash());
    EXPECT_EQ(tree.get_leaves()[1].hash(), WrappedNullifierLeaf({ 30, 0, 0 }).hash());
    EXPECT_EQ(tree.get_leaves()[2].hash(), WrappedNullifierLeaf({ 10, 3, 20 }).hash());
    EXPECT_EQ(tree.get_leaves()[3].hash(), WrappedNullifierLeaf({ 20, 1, 30 }).hash());

    /**
     * Add new value 50:
     *
     *  index     0       1       2       3        4       5       6       7
     *  ---------------------------------------------------------------------
     *  val       0       30      10      20       50      0       0       0
     *  nextIdx   2       4       3       1        0       0       0       0
     *  nextVal   10      50      20      30       0       0       0       0
     */
    tree.update_element(50);
    EXPECT_EQ(tree.get_leaves().size(), 5);
    EXPECT_EQ(tree.get_leaves()[0].hash(), WrappedNullifierLeaf({ 0, 2, 10 }).hash());
    EXPECT_EQ(tree.get_leaves()[1].hash(), WrappedNullifierLeaf({ 30, 4, 50 }).hash());
    EXPECT_EQ(tree.get_leaves()[2].hash(), WrappedNullifierLeaf({ 10, 3, 20 }).hash());
    EXPECT_EQ(tree.get_leaves()[3].hash(), WrappedNullifierLeaf({ 20, 1, 30 }).hash());
    EXPECT_EQ(tree.get_leaves()[4].hash(), WrappedNullifierLeaf({ 50, 0, 0 }).hash());

    // Manually compute the node values
    auto e000 = tree.get_leaves()[0].hash();
    auto e001 = tree.get_leaves()[1].hash();
    auto e010 = tree.get_leaves()[2].hash();
    auto e011 = tree.get_leaves()[3].hash();
    auto e100 = tree.get_leaves()[4].hash();
    auto e101 = WrappedNullifierLeaf::zero().hash();
    auto e110 = WrappedNullifierLeaf::zero().hash();
    auto e111 = WrappedNullifierLeaf::zero().hash();

    auto e00 = hash_pair_native(e000, e001);
    auto e01 = hash_pair_native(e010, e011);
    auto e10 = hash_pair_native(e100, e101);
    auto e11 = hash_pair_native(e110, e111);

    auto e0 = hash_pair_native(e00, e01);
    auto e1 = hash_pair_native(e10, e11);
    auto root = hash_pair_native(e0, e1);

    // Check the hash path at index 2 and 3
    // Note: This merkle proof would also serve as a non-membership proof of values in (10, 20) and (20, 30)
    fr_hash_path expected = {
        std::make_pair(e010, e011),
        std::make_pair(e00, e01),
        std::make_pair(e0, e1),
    };
    EXPECT_EQ(tree.get_hash_path(2), expected);
    EXPECT_EQ(tree.get_hash_path(3), expected);
    EXPECT_EQ(tree.root(), root);

    // Check the hash path at index 6 and 7
    expected = {
        std::make_pair(e110, e111),
        std::make_pair(e10, e11),
        std::make_pair(e0, e1),
    };
    EXPECT_EQ(tree.get_hash_path(6), expected);
    EXPECT_EQ(tree.get_hash_path(7), expected);
}

TEST(crypto_nullifier_tree, test_nullifier_memory_appending_zero)
{
    // Create a depth-3 indexed merkle tree
    constexpr size_t depth = 3;
    NullifierMemoryTree tree(depth);

    /**
     * Intial state:
     *
     *  index     0       1       2       3        4       5       6       7
     *  ---------------------------------------------------------------------
     *  val       0       0       0       0        0       0       0       0
     *  nextIdx   0       0       0       0        0       0       0       0
     *  nextVal   0       0       0       0        0       0       0       0
     */
    WrappedNullifierLeaf zero_leaf = WrappedNullifierLeaf({ 0, 0, 0 });
    EXPECT_EQ(tree.get_leaves().size(), 1);
    EXPECT_EQ(tree.get_leaves()[0], zero_leaf);

    /**
     * Add new value 30:
     *
     *  index     0       1       2       3        4       5       6       7
     *  ---------------------------------------------------------------------
     *  val       0       30      0       0        0       0       0       0
     *  nextIdx   1       0       0       0        0       0       0       0
     *  nextVal   30      0       0       0        0       0       0       0
     */
    tree.update_element(30);
    EXPECT_EQ(tree.get_leaves().size(), 2);
    EXPECT_EQ(tree.get_leaves()[0].hash(), WrappedNullifierLeaf({ 0, 1, 30 }).hash());
    EXPECT_EQ(tree.get_leaves()[1].hash(), WrappedNullifierLeaf({ 30, 0, 0 }).hash());

    /**
     * Add new value 10:
     *
     *  index     0       1       2       3        4       5       6       7
     *  ---------------------------------------------------------------------
     *  val       0       30      10      0        0       0       0       0
     *  nextIdx   2       0       1       0        0       0       0       0
     *  nextVal   10      0       30      0        0       0       0       0
     */
    tree.update_element(10);
    EXPECT_EQ(tree.get_leaves().size(), 3);
    EXPECT_EQ(tree.get_leaves()[0].hash(), WrappedNullifierLeaf({ 0, 2, 10 }).hash());
    EXPECT_EQ(tree.get_leaves()[1].hash(), WrappedNullifierLeaf({ 30, 0, 0 }).hash());
    EXPECT_EQ(tree.get_leaves()[2].hash(), WrappedNullifierLeaf({ 10, 1, 30 }).hash());

    /**
     * Add new value 20:
     *
     *  index     0       1       2       3        4       5       6       7
     *  ---------------------------------------------------------------------
     *  val       0       30      10      20       0       0       0       0
     *  nextIdx   2       0       3       1        0       0       0       0
     *  nextVal   10      0       20      30       0       0       0       0
     */
    tree.update_element(20);
    EXPECT_EQ(tree.get_leaves().size(), 4);
    EXPECT_EQ(tree.get_leaves()[0].hash(), WrappedNullifierLeaf({ 0, 2, 10 }).hash());
    EXPECT_EQ(tree.get_leaves()[1].hash(), WrappedNullifierLeaf({ 30, 0, 0 }).hash());
    EXPECT_EQ(tree.get_leaves()[2].hash(), WrappedNullifierLeaf({ 10, 3, 20 }).hash());
    EXPECT_EQ(tree.get_leaves()[3].hash(), WrappedNullifierLeaf({ 20, 1, 30 }).hash());

    // Adding the same value must not affect anything
    tree.update_element(20);
    EXPECT_EQ(tree.get_leaves().size(), 4);
    EXPECT_EQ(tree.get_leaves()[0].hash(), WrappedNullifierLeaf({ 0, 2, 10 }).hash());
    EXPECT_EQ(tree.get_leaves()[1].hash(), WrappedNullifierLeaf({ 30, 0, 0 }).hash());
    EXPECT_EQ(tree.get_leaves()[2].hash(), WrappedNullifierLeaf({ 10, 3, 20 }).hash());
    EXPECT_EQ(tree.get_leaves()[3].hash(), WrappedNullifierLeaf({ 20, 1, 30 }).hash());

    /**
     * Add new value 0:
     *
     *  index     0       1       2       3        4       5       6       7
     *  ---------------------------------------------------------------------
     *  val       0       30      10      20       0       0       0       0
     *  nextIdx   2       0       3       1        0       0       0       0
     *  nextVal   10      0       20      30       0       0       0       0
     */
    tree.update_element(0);
    EXPECT_EQ(tree.get_leaves().size(), 5);
    EXPECT_EQ(tree.get_leaves()[0].hash(), WrappedNullifierLeaf({ 0, 2, 10 }).hash());
    EXPECT_EQ(tree.get_leaves()[1].hash(), WrappedNullifierLeaf({ 30, 0, 0 }).hash());
    EXPECT_EQ(tree.get_leaves()[2].hash(), WrappedNullifierLeaf({ 10, 3, 20 }).hash());
    EXPECT_EQ(tree.get_leaves()[3].hash(), WrappedNullifierLeaf({ 20, 1, 30 }).hash());
    EXPECT_EQ(tree.get_leaves()[4].hash(), WrappedNullifierLeaf::zero().hash());

    /*
     * Add new value 0:
     *
     *  index     0       1       2       3        4       5       6       7
     *  ---------------------------------------------------------------------
     *  val       0       30      10      20       0       0       0       0
     *  nextIdx   2       0       3       1        0       0       0       0
     *  nextVal   10      0       20      30       0       0       0       0
     */
    tree.update_element(0);
    EXPECT_EQ(tree.get_leaves().size(), 6);
    EXPECT_EQ(tree.get_leaves()[0].hash(), WrappedNullifierLeaf({ 0, 2, 10 }).hash());
    EXPECT_EQ(tree.get_leaves()[1].hash(), WrappedNullifierLeaf({ 30, 0, 0 }).hash());
    EXPECT_EQ(tree.get_leaves()[2].hash(), WrappedNullifierLeaf({ 10, 3, 20 }).hash());
    EXPECT_EQ(tree.get_leaves()[3].hash(), WrappedNullifierLeaf({ 20, 1, 30 }).hash());
    EXPECT_EQ(tree.get_leaves()[4].hash(), WrappedNullifierLeaf::zero().hash());
    EXPECT_EQ(tree.get_leaves()[5].hash(), WrappedNullifierLeaf::zero().hash());

    /**
     * Add new value 50:
     *
     *  index     0       1       2       3        4       5       6       7
     *  ---------------------------------------------------------------------
     *  val       0       30      10      20       0       0      50       0
     *  nextIdx   2       6       3       1        0       0       0       0
     *  nextVal   10      50      20      30       0       0       0       0
     */
    tree.update_element(50);
    EXPECT_EQ(tree.get_leaves().size(), 7);
    EXPECT_EQ(tree.get_leaf(0).hash(), WrappedNullifierLeaf({ 0, 2, 10 }).hash());
    EXPECT_EQ(tree.get_leaf(1).hash(), WrappedNullifierLeaf({ 30, 6, 50 }).hash());
    EXPECT_EQ(tree.get_leaf(2).hash(), WrappedNullifierLeaf({ 10, 3, 20 }).hash());
    EXPECT_EQ(tree.get_leaf(3).hash(), WrappedNullifierLeaf({ 20, 1, 30 }).hash());
    EXPECT_EQ(tree.get_leaf(4).hash(), WrappedNullifierLeaf::zero().hash());
    EXPECT_EQ(tree.get_leaf(5).hash(), WrappedNullifierLeaf::zero().hash());
    EXPECT_EQ(tree.get_leaf(6).hash(), WrappedNullifierLeaf({ 50, 0, 0 }).hash());
    EXPECT_EQ(tree.get_leaf(7).hash(), WrappedNullifierLeaf::zero().hash());

    // Manually compute the node values
    auto e000 = tree.get_leaf(0).hash();
    auto e001 = tree.get_leaf(1).hash();
    auto e010 = tree.get_leaf(2).hash();
    auto e011 = tree.get_leaf(3).hash();
    auto e100 = tree.get_leaf(4).hash();
    auto e101 = tree.get_leaf(5).hash();
    auto e110 = tree.get_leaf(6).hash();
    auto e111 = tree.get_leaf(7).hash();

    auto e00 = hash_pair_native(e000, e001);
    auto e01 = hash_pair_native(e010, e011);
    auto e10 = hash_pair_native(e100, e101);
    auto e11 = hash_pair_native(e110, e111);

    auto e0 = hash_pair_native(e00, e01);
    auto e1 = hash_pair_native(e10, e11);
    auto root = hash_pair_native(e0, e1);

    // Check the hash path at index 2 and 3
    // Note: This merkle proof would also serve as a non-membership proof of values in (10, 20) and (20, 30)
    fr_hash_path expected = {
        std::make_pair(e010, e011),
        std::make_pair(e00, e01),
        std::make_pair(e0, e1),
    };
    EXPECT_EQ(tree.get_hash_path(2), expected);
    EXPECT_EQ(tree.get_hash_path(3), expected);
    EXPECT_EQ(tree.root(), root);

    // Check the hash path at index 6 and 7
    expected = {
        std::make_pair(e110, e111),
        std::make_pair(e10, e11),
        std::make_pair(e0, e1),
    };
    EXPECT_EQ(tree.get_hash_path(6), expected);
    EXPECT_EQ(tree.get_hash_path(7), expected);
}
TEST(crypto_nullifier_tree, test_nullifier_tree)
{
    // Create a depth-8 indexed merkle tree
    constexpr size_t depth = 8;
    NullifierMemoryTree tree(depth);

    nullifier_leaf zero_leaf = { 0, 0, 0 };
    EXPECT_EQ(tree.get_leaves().size(), 1);
    EXPECT_EQ(tree.get_leaves()[0].hash(), zero_leaf.hash());

    // Add 20 random values to the tree
    for (size_t i = 0; i < 20; i++) {
        auto value = fr::random_element();
        tree.update_element(value);
    }

    auto abs_diff = [](uint256_t a, uint256_t b) {
        if (a > b) {
            return (a - b);
        } else {
            return (b - a);
        }
    };

    // Check if a new random value is not a member of this tree.
    fr new_member = fr::random_element();
    const auto& leaves = tree.get_leaves();
    std::vector<uint256_t> differences;
    for (size_t i = 0; i < leaves.size(); i++) {
        uint256_t diff_hi =
            abs_diff(uint256_t(new_member), uint256_t(leaves[i].has_value() ? leaves[i].unwrap().value : 0));
        uint256_t diff_lo =
            abs_diff(uint256_t(new_member), uint256_t(leaves[i].has_value() ? leaves[i].unwrap().nextValue : 0));
        differences.push_back(diff_hi + diff_lo);
    }
    auto it = std::min_element(differences.begin(), differences.end());
    auto index = static_cast<size_t>(it - differences.begin());

    // Merkle proof at `index` proves non-membership of `new_member`
    auto hash_path = tree.get_hash_path(index);
    EXPECT_TRUE(check_hash_path(tree.root(), hash_path, leaves[index].unwrap(), index));
}