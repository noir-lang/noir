#include "compute_rollup_circuit_data.hpp"
#include "create_noop_join_split_proof.hpp"
#include "verify_rollup.hpp"
#include <gtest/gtest.h>

using namespace barretenberg;
using namespace rollup::rollup_proofs;

TEST(rollup_proofs, test_rollup_1_proofs)
{
    size_t rollup_size = 1;
    auto join_split_proof = create_noop_join_split_proof();
    auto rollup_circuit_data = compute_rollup_circuit_data(rollup_size);

    auto gibberish_data_path = fr_hash_path(32, std::make_pair(fr::random_element(), fr::random_element()));
    auto gibberish_null_path = fr_hash_path(128, std::make_pair(fr::random_element(), fr::random_element()));
    rollup_tx rollup = {
        0,
        (uint32_t)rollup_size,
        (uint32_t)join_split_proof.proof_data.size(),
        0,
        std::vector(rollup_size, join_split_proof.proof_data),
        fr::random_element(),
        fr::random_element(),
        std::vector(rollup_size * 2, std::make_pair(uint128_t(0), gibberish_data_path)),
        std::vector(rollup_size * 2, std::make_pair(uint128_t(0), gibberish_data_path)),
        fr::random_element(),
        fr::random_element(),
        std::vector(rollup_size * 2, std::make_pair(uint128_t(0), gibberish_null_path)),
        std::vector(rollup_size * 2, std::make_pair(uint128_t(0), gibberish_null_path)),
    };

    auto verified = verify_rollup(rollup, rollup_circuit_data);

    EXPECT_TRUE(verified);
}