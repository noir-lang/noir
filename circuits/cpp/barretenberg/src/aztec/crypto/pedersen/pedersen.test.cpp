#include <gtest/gtest.h>
#include <common/streams.hpp>
#include "./pedersen.hpp"

using namespace crypto::pedersen;

TEST(pedersen, hash_ladder_structure)
{
    generator_index_t index = { 2, 0 };
    generator_data gen_data = get_generator_data(index);
    auto P = grumpkin::g1::element(gen_data.generator);
    auto Q = grumpkin::g1::element(gen_data.aux_generator);

    /**
     * Check if the hash ladder is structured in the following way:
     * +-----+------------+----------------+
     * | idx | one        | three          |
     * +-----+------------+----------------+
     * | 0   | 4^{n-2}[P] | (3*4^{n-2})[P] |
     * | 1   | 4^{n-3}[P] | (3*4^{n-3})[P] |
     * | 2   | 4^{n-4}[P] | (3*4^{n-4})[P] |
     * | .   | .          | .              |
     * | .   | .          | .              |
     * | .   | .          | .              |
     * | 124 | 4[P]       | (3*4)[P]       |
     * | 125 | 1[P]       | (3*1)[P]       |
     * +-----+------------+----------------+
     * | 126 | 4[Q]       | (3*4)[Q]       |
     * | 127 | 1[Q]       | (3*1)[Q]       |
     * +-----+------------+----------------+
     *
     * Here num_quads is n = 127.
     */
    const uint32_t num_quads = 127;
    auto hash_ladder = gen_data.get_hash_ladder(254);

    // Check auxiliary generator powers
    grumpkin::g1::element acc_q = Q;
    for (size_t i = num_quads; i > (num_quads - 2); i--) {
        auto local_acc_q = acc_q;
        EXPECT_EQ(acc_q, grumpkin::g1::element(hash_ladder[i].one));
        acc_q.self_dbl();
        EXPECT_EQ((acc_q + local_acc_q), grumpkin::g1::element(hash_ladder[i].three));
        acc_q.self_dbl();
    }

    // Check normal generator powers
    grumpkin::g1::element acc_p = P;
    for (int i = num_quads - 2; i >= 0; i--) {
        auto local_acc_p = acc_p;
        EXPECT_EQ(acc_p, grumpkin::g1::element(hash_ladder[i].one));
        acc_p.self_dbl();
        EXPECT_EQ((acc_p + local_acc_p), grumpkin::g1::element(hash_ladder[i].three));
        acc_p.self_dbl();
    }

    // Check the 0-th value in hash ladder.
    const auto scalar = grumpkin::fq(uint256_t(1) << 250);
    const auto mult = fixed_base_scalar_mul<254>(barretenberg::fr(scalar), 2);
    EXPECT_EQ(grumpkin::g1::element(hash_ladder[0].one), mult);
}

TEST(pedersen, fixed_base_scalar_mul)
{
    uint256_t scalar(123, 0, 0, 0);

    grumpkin::fr priv_key(scalar);
    generator_index_t index = { 0, 0 };
    auto pub_key = get_generator_data(index).generator * priv_key;
    auto result = fixed_base_scalar_mul<128>(barretenberg::fr(scalar), 0);

    EXPECT_EQ(result.x, pub_key.x);
    EXPECT_EQ(result.y, pub_key.y);

    {
        uint256_t scalar(123, 523, 0, 0);
        grumpkin::fr priv_key(scalar);
        generator_index_t index = { 5, 0 };
        auto gen_data = get_generator_data(index);
        auto pub_key = gen_data.generator * priv_key;
        auto result = fixed_base_scalar_mul<128>(barretenberg::fr(scalar), 5);

        EXPECT_EQ(result.x, pub_key.x);
        EXPECT_EQ(result.y, pub_key.y);
    }
}

TEST(pedersen, compress_zero)
{
    grumpkin::fq zero(0);
    auto result = compress_native({ zero, zero });
    EXPECT_EQ(result, 0);
}
