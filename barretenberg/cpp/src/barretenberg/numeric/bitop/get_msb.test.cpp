#include "get_msb.hpp"
#include <gtest/gtest.h>

using namespace bb;

TEST(bitop, GetMsbUint640Value)
{
    uint64_t a = 0b00000000000000000000000000000000;
    EXPECT_EQ(numeric::get_msb(a), 0U);
}

TEST(bitop, GetMsbUint320)
{
    uint32_t a = 0b00000000000000000000000000000001;
    EXPECT_EQ(numeric::get_msb(a), 0U);
}

TEST(bitop, GetMsbUint3231)
{
    uint32_t a = 0b10000000000000000000000000000001;
    EXPECT_EQ(numeric::get_msb(a), 31U);
}

TEST(bitop, GetMsbUint6463)
{
    uint64_t a = 0b1000000000000000000000000000000100000000000000000000000000000000;
    EXPECT_EQ(numeric::get_msb(a), 63U);
}

TEST(bitop, GetMsbSizeT7)
{
    size_t a = 0x80;
    auto r = numeric::get_msb(a);
    EXPECT_EQ(r, 7U);
}
