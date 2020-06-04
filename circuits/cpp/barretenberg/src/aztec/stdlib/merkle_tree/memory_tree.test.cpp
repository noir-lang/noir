#include "memory_tree.hpp"
#include <gtest/gtest.h>
#include <stdlib/types/turbo.hpp>

using namespace barretenberg;
using namespace plonk::stdlib::merkle_tree;

static std::vector<std::vector<uint8_t>> VALUES = []() {
    std::vector<std::vector<uint8_t>> values(4);
    for (size_t i = 0; i < 4; ++i) {
        std::vector<uint8_t> v(64, 0);
        *(size_t*)v.data() = i;
        values[i] = v;
    }
    return values;
}();

TEST(stdlib_merkle_tree, test_memory_store)
{
    fr e00 = hash_value_native(VALUES[0]);
    fr e01 = hash_value_native(VALUES[1]);
    fr e02 = hash_value_native(VALUES[2]);
    fr e03 = hash_value_native(VALUES[3]);
    fr e10 = compress_native({ e00, e01 });
    fr e11 = compress_native({ e02, e03 });
    fr root = compress_native({ e10, e11 });

    MemoryTree db(2);
    for (size_t i = 0; i < 4; ++i) {
        db.update_element(i, VALUES[i]);
    }
    for (size_t i = 0; i < 4; ++i) {
        EXPECT_EQ(db.get_element(i), VALUES[i]);
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