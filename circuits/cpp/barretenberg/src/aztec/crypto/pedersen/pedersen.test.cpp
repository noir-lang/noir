#include <gtest/gtest.h>
#include "./pedersen.hpp"

TEST(pedersen, fixed_base_scalar_mul)
{
    uint256_t scalar(123, 0, 0, 0);

    grumpkin::fr priv_key(scalar);
    auto pub_key = crypto::pedersen::get_generator(0) * priv_key;
    auto result = crypto::pedersen::fixed_base_scalar_mul<128>(barretenberg::fr(scalar), 0);

    EXPECT_EQ(result.x, pub_key.x);
    EXPECT_EQ(result.y, pub_key.y);
}