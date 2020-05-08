#include "rollup_tx.hpp"
#include <gtest/gtest.h>

using namespace rollup::rollup_proofs;
using namespace barretenberg;

TEST(rollup_tx, test_serialization)
{
    rollup_tx rollup;
    rollup.rollup_id = 5;
    rollup.num_txs = 3;
    rollup.proof_lengths = 123;
    rollup.txs = std::vector(rollup.num_txs, std::vector<uint8_t>(rollup.proof_lengths, 0x80));
    rollup.old_data_root = fr::random_element();
    rollup.old_null_root = fr::random_element();
    auto random_pair = std::make_pair(fr::random_element(), fr::random_element());
    rollup.old_data_paths =
        std::vector(rollup.num_txs, std::array{ fr_hash_path(32, random_pair), fr_hash_path(32, random_pair) });
    rollup.old_null_paths =
        std::vector(rollup.num_txs, std::array{ fr_hash_path(128, random_pair), fr_hash_path(128, random_pair) });
    rollup.new_data_root = fr::random_element();
    rollup.new_null_root = fr::random_element();
    rollup.new_data_paths =
        std::vector(rollup.num_txs, std::array{ fr_hash_path(32, random_pair), fr_hash_path(32, random_pair) });
    rollup.new_null_paths =
        std::vector(rollup.num_txs, std::array{ fr_hash_path(128, random_pair), fr_hash_path(128, random_pair) });

    auto buf = to_buffer(rollup);
    auto result = from_buffer<rollup_tx>(buf);

    EXPECT_EQ(rollup, result);
}