#include "rollup_tx.hpp"
#include <gtest/gtest.h>

using namespace rollup::rollup_proofs;

TEST(rollup_tx, test_serialization)
{
    rollup_tx rollup;
    rollup.rollup_id = 5;
    rollup.num_txs = 3;
    rollup.proof_lengths = 123;
    rollup.txs = std::vector(rollup.num_txs, std::vector<uint8_t>(rollup.proof_lengths, 0x80));

    auto buf = to_buffer(rollup);
    auto result = from_buffer<rollup_tx>(buf);

    EXPECT_EQ(rollup, result);
}