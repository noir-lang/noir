#include "compute_circuit_data.hpp"
#include <gtest/gtest.h>

using namespace rollup::proofs::escape_hatch;

TEST(escape_hatch_tx_tests, serialization)
{
    auto tx = dummy_tx();

    auto buffer = to_buffer(tx);
    auto tx2 = from_buffer<escape_hatch_tx>(buffer.data());

    EXPECT_EQ(tx, tx2);
}
