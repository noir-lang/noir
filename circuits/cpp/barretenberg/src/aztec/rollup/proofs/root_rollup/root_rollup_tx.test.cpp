#include "root_rollup_tx.hpp"
#include "../../constants.hpp"
#include <gtest/gtest.h>

using namespace rollup::proofs::root_rollup;
using namespace barretenberg;

TEST(root_rollup_tx, test_serialization)
{
    auto random_pair = std::make_pair(fr::random_element(), fr::random_element());

    root_rollup_tx rollup;
    rollup.num_inner_proofs = 2;
    rollup.rollup_id = 5;
    rollup.rollups = std::vector(2, std::vector<uint8_t>(123, 0x80));

    rollup.old_data_roots_root = fr::random_element();
    rollup.new_data_roots_root = fr::random_element();
    rollup.old_data_roots_path = fr_hash_path(28, random_pair);
    rollup.new_data_roots_path = fr_hash_path(28, random_pair);

    auto buf = to_buffer(rollup);
    auto result = from_buffer<root_rollup_tx>(buf);

    EXPECT_EQ(result.num_inner_proofs, rollup.num_inner_proofs);
    EXPECT_EQ(result.rollup_id, rollup.rollup_id);
    EXPECT_EQ(result.old_data_roots_root, rollup.old_data_roots_root);
    EXPECT_EQ(result.new_data_roots_root, rollup.new_data_roots_root);
    EXPECT_EQ(result.old_data_roots_path, rollup.old_data_roots_path);
    EXPECT_EQ(result.new_data_roots_path, rollup.new_data_roots_path);
}
