#include "compute_rollup_circuit_data.hpp"
#include "create_noop_join_split_proof.hpp"
#include "verify_rollup.hpp"
#include <gtest/gtest.h>

using namespace rollup::rollup_proofs;

TEST(rollup_proofs, test_rollup_1_proofs)
{
    size_t rollup_size = 1;
    auto join_split_proof = create_noop_join_split_proof();
    auto rollup_circuit_data = compute_rollup_circuit_data(rollup_size);

    auto verified = verify_rollup(std::vector(rollup_size, join_split_proof), rollup_circuit_data);

    EXPECT_TRUE(verified);
}