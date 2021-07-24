#include <gtest/gtest.h>
#include <common/streams.hpp>
#include <common/map.hpp>
#include "./pedersen.hpp"

using namespace crypto::pedersen;

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