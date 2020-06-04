#include "rollup_tx.hpp"
#include <gtest/gtest.h>

using namespace rollup::rollup_proofs;
using namespace barretenberg;

TEST(rollup_tx, test_serialization)
{
    auto random_pair = std::make_pair(fr::random_element(), fr::random_element());

    rollup_tx rollup;
    rollup.rollup_id = 5;
    rollup.num_txs = 3;
    rollup.data_start_index = 0;
    rollup.txs = std::vector(rollup.num_txs, std::vector<uint8_t>(123, 0x80));

    rollup.rollup_root = fr::random_element();
    rollup.old_data_root = fr::random_element();
    rollup.new_data_root = fr::random_element();
    rollup.old_data_path = fr_hash_path(32, random_pair);
    rollup.new_data_path = fr_hash_path(32, random_pair);

    rollup.old_null_root = fr::random_element();
    rollup.new_null_roots = std::vector(rollup.num_txs * 2, fr::random_element());
    rollup.old_null_paths = std::vector(rollup.num_txs * 2, fr_hash_path(128, random_pair));
    rollup.new_null_paths = std::vector(rollup.num_txs * 2, fr_hash_path(128, random_pair));

    rollup.data_roots_root = fr::random_element();
    rollup.data_roots_paths = std::vector(rollup.num_txs, fr_hash_path(28, random_pair));
    rollup.data_roots_indicies = std::vector(rollup.num_txs, 0U);

    auto buf = to_buffer(rollup);
    auto result = from_buffer<rollup_tx>(buf);

    EXPECT_EQ(result.rollup_id, rollup.rollup_id);
    EXPECT_EQ(result.num_txs, rollup.num_txs);
    EXPECT_EQ(result.data_start_index, rollup.data_start_index);
    EXPECT_EQ(result.txs, rollup.txs);

    EXPECT_EQ(result.rollup_root, rollup.rollup_root);
    EXPECT_EQ(result.old_data_root, rollup.old_data_root);
    EXPECT_EQ(result.new_data_root, rollup.new_data_root);
    EXPECT_EQ(result.old_data_path, rollup.old_data_path);
    EXPECT_EQ(result.new_data_path, rollup.new_data_path);

    EXPECT_EQ(result.old_null_root, rollup.old_null_root);
    EXPECT_EQ(result.new_null_roots, rollup.new_null_roots);
    EXPECT_EQ(result.old_null_paths, rollup.old_null_paths);
    EXPECT_EQ(result.new_null_paths, rollup.new_null_paths);

    EXPECT_EQ(result.data_roots_root, rollup.data_roots_root);
    EXPECT_EQ(result.data_roots_paths, rollup.data_roots_paths);
    EXPECT_EQ(result.data_roots_indicies, rollup.data_roots_indicies);
}
