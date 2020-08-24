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

TEST(engine, reset_debug_engine)
{
    auto& debug_engine = numeric::random::get_debug_engine();

    auto a = debug_engine.get_random_uint64();
    auto b = debug_engine.get_random_uint64();
    EXPECT_NE(a, b);

    debug_engine = numeric::random::get_debug_engine(true);
    auto c = debug_engine.get_random_uint64();
    auto d = debug_engine.get_random_uint64();
    EXPECT_EQ(a, c);
    EXPECT_EQ(b, d);

    auto e = numeric::random::get_engine().get_random_uint64();
    EXPECT_NE(a, e);
}
