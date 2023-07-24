#include "barretenberg/ecc/fields/field.hpp"
#include "barretenberg/serialize/msgpack.hpp"
#include "barretenberg/serialize/test_helper.hpp"
#include <gtest/gtest.h>

TEST(msgpack_tests, msgpack_field)
{
    auto [actual, expected] = msgpack_roundtrip(barretenberg::fr{ 1ull, 2ull, 3ull, 4ull });
    EXPECT_EQ(actual, expected);
}
