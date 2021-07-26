#include "rollup_tx.hpp"
#include "../../constants.hpp"
#include <gtest/gtest.h>

using namespace rollup::proofs::rollup;
using namespace barretenberg;

TEST(rollup_tx, test_serialization)
{
    auto random_pair = std::make_pair(fr::random_element(), fr::random_element());

    rollup_tx rollup;
    rollup.rollup_id = 1;
    rollup.num_txs = 3;
    rollup.data_start_index = 0;
    rollup.txs = std::vector(rollup.num_txs, std::vector<uint8_t>(123, 0x80));

    rollup.old_data_root = fr::random_element();
    rollup.new_data_root = fr::random_element();
    rollup.old_data_path = fr_hash_path(32, random_pair);

    rollup.old_null_root = fr::random_element();
    rollup.new_null_roots = std::vector(rollup.num_txs * 2, fr::random_element());
    rollup.old_null_paths = std::vector(rollup.num_txs * 2, fr_hash_path(rollup::NULL_TREE_DEPTH, random_pair));

    rollup.data_roots_root = fr::random_element();
    rollup.data_roots_paths = std::vector(rollup.num_txs, fr_hash_path(28, random_pair));
    rollup.data_roots_indicies = std::vector(rollup.num_txs, 0U);

    rollup.new_defi_root = fr::random_element();
    rollup.bridge_ids = { 0, 1, 2, 3 };
    rollup.asset_ids = { 4, 5, 6, 7 };

    auto buf = to_buffer(rollup);
    auto result = from_buffer<rollup_tx>(buf);

    EXPECT_EQ(result.rollup_id, rollup.rollup_id);
    EXPECT_EQ(result.num_txs, rollup.num_txs);
    EXPECT_EQ(result.data_start_index, rollup.data_start_index);
    EXPECT_EQ(result.txs, rollup.txs);

    EXPECT_EQ(result.old_data_root, rollup.old_data_root);
    EXPECT_EQ(result.new_data_root, rollup.new_data_root);
    EXPECT_EQ(result.old_data_path, rollup.old_data_path);

    EXPECT_EQ(result.old_null_root, rollup.old_null_root);
    EXPECT_EQ(result.new_null_roots, rollup.new_null_roots);
    EXPECT_EQ(result.old_null_paths, rollup.old_null_paths);

    EXPECT_EQ(result.data_roots_root, rollup.data_roots_root);
    EXPECT_EQ(result.data_roots_paths, rollup.data_roots_paths);
    EXPECT_EQ(result.data_roots_indicies, rollup.data_roots_indicies);

    EXPECT_EQ(result.new_defi_root, rollup.new_defi_root);
    EXPECT_EQ(result.bridge_ids, rollup.bridge_ids);
    EXPECT_EQ(result.asset_ids, rollup.asset_ids);
}
