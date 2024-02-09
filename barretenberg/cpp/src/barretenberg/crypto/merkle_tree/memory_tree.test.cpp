#include "memory_tree.hpp"
#include <gtest/gtest.h>

using namespace bb;
using namespace bb::crypto::merkle_tree;

using HashPolicy = PedersenHashPolicy;

static std::vector<fr> VALUES = []() {
    std::vector<fr> values(4);
    for (size_t i = 0; i < 4; ++i) {
        values[i] = fr(i);
    }
    return values;
}();

TEST(crypto_merkle_tree, test_memory_store)
{
    fr e00 = 0;
    fr e01 = VALUES[1];
    fr e02 = VALUES[2];
    fr e03 = VALUES[3];
    fr e10 = HashPolicy::hash_pair(e00, e01);
    fr e11 = HashPolicy::hash_pair(e02, e03);
    fr root = HashPolicy::hash_pair(e10, e11);

    MemoryTree<HashPolicy> db(2);
    for (size_t i = 0; i < 4; ++i) {
        db.update_element(i, VALUES[i]);
    }
    fr_hash_path expected = {
        std::make_pair(e00, e01),
        std::make_pair(e10, e11),
    };
    EXPECT_EQ(db.get_hash_path(0), expected);
    EXPECT_EQ(db.get_hash_path(1), expected);

    expected = {
        std::make_pair(e02, e03),
        std::make_pair(e10, e11),
    };

    EXPECT_EQ(db.get_hash_path(2), expected);
    EXPECT_EQ(db.get_hash_path(3), expected);
    EXPECT_EQ(db.root(), root);
}

TEST(crypto_merkle_tree, test_memory_store_sibling_path)
{
    fr e00 = 0;
    fr e01 = VALUES[1];
    fr e02 = VALUES[2];
    fr e03 = VALUES[3];
    fr e10 = HashPolicy::hash_pair(e00, e01);
    fr e11 = HashPolicy::hash_pair(e02, e03);
    fr root = HashPolicy::hash_pair(e10, e11);

    MemoryTree<HashPolicy> db(2);
    for (size_t i = 0; i < 4; ++i) {
        db.update_element(i, VALUES[i]);
    }

    // Check correct paths are generated for each layer 0 element
    fr_sibling_path expected00 = {
        e01,
        e11,
    };
    fr_sibling_path expected01 = { e00, e11 };
    fr_sibling_path expected02 = {
        e03,
        e10,
    };
    fr_sibling_path expected03 = {
        e02,
        e10,
    };
    EXPECT_EQ(db.get_sibling_path(0), expected00);
    EXPECT_EQ(db.get_sibling_path(1), expected01);
    EXPECT_EQ(db.get_sibling_path(2), expected02);
    EXPECT_EQ(db.get_sibling_path(3), expected03);
    EXPECT_EQ(db.root(), root);
}
