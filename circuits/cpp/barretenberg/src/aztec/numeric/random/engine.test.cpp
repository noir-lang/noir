#include "engine.hpp"
#include <gtest/gtest.h>

TEST(engine, get_random_uint64)
{
    auto& engine = numeric::random::get_engine();
    auto a = engine.get_random_uint64();
    auto b = engine.get_random_uint64();
    EXPECT_NE(a, 0U);
    EXPECT_NE(b, 0U);
    EXPECT_NE(a, b);
}
