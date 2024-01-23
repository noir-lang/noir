#include "count_leading_zeros.hpp"
#include <gtest/gtest.h>

using namespace bb;

TEST(bitop, ClzUint3231)
{
    uint32_t a = 0b00000000000000000000000000000001;
    EXPECT_EQ(numeric::count_leading_zeros(a), 31U);
}

TEST(bitop, ClzUint320)
{
    uint32_t a = 0b10000000000000000000000000000001;
    EXPECT_EQ(numeric::count_leading_zeros(a), 0U);
}

TEST(bitop, ClzUint640)
{
    uint64_t a = 0b1000000000000000000000000000000100000000000000000000000000000000;
    EXPECT_EQ(numeric::count_leading_zeros(a), 0U);
}

TEST(bitop, ClzUint256255)
{
    uint256_t a = 0x1;
    auto r = numeric::count_leading_zeros(a);
    EXPECT_EQ(r, 255U);
}

TEST(bitop, ClzUint256248)
{
    uint256_t a = 0x80;
    auto r = numeric::count_leading_zeros(a);
    EXPECT_EQ(r, 248U);
}
