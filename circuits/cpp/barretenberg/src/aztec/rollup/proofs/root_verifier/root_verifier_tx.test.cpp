#include "root_verifier_tx.hpp"
#include <gtest/gtest.h>

using namespace rollup::proofs::root_verifier;

TEST(RootVerifierTransaction, Serialization)
{
    root_verifier_tx tx;
    tx.broadcast_data = std::vector<uint8_t>(66, 0xf);
    tx.proof_data = std::vector<uint8_t>(123, 0x80);
    auto buf = to_buffer(tx);
    auto result = from_buffer<root_verifier_tx>(buf);
    EXPECT_EQ(result.broadcast_data, tx.broadcast_data);
    EXPECT_EQ(result.proof_data, tx.proof_data);
}
