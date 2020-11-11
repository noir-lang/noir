#include "count_leading_zeros.hpp"
#include <gtest/gtest.h>

TEST(bitop, clz_uint32_0)
{
    uint32_t a = 0b00000000000000000000000000000001;
    EXPECT_EQ(numeric::count_leading_zeros(a), 31U);
}

TEST(bitop, clz_uint32_31)
{
    uint32_t a = 0b10000000000000000000000000000001;
    EXPECT_EQ(numeric::count_leading_zeros(a), 0U);
}

TEST(bitop, clz_uint64_63)
{
    uint64_t a = 0b1000000000000000000000000000000100000000000000000000000000000000;
    EXPECT_EQ(numeric::count_leading_zeros(a), 0U);
}

TEST(bitop, clz_size_t_7)
{
    size_t a = 0x80;
    auto r = numeric::count_leading_zeros(a);
    EXPECT_EQ(r, 56U);
}

TEST(bitop, clz_uint256_255)
{
    uint256_t a = 0x1;
    auto r = numeric::count_leading_zeros(a);
    EXPECT_EQ(r, 255U);
}

TEST(bitop, clz_uint256_7)
{
    uint256_t a = 0x80;
    auto r = numeric::count_leading_zeros(a);
    EXPECT_EQ(r, 248U);
}