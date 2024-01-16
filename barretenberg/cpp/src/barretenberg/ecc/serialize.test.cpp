#include "barretenberg/ecc/fields/field.hpp"
#include "barretenberg/serialize/msgpack.hpp"
#include "barretenberg/serialize/test_helper.hpp"
#include <gtest/gtest.h>

TEST(MsgpackTests, MsgpackField)
{
    auto [actual, expected] = msgpack_roundtrip(bb::fr{ 1ULL, 2ULL, 3ULL, 4ULL });
    EXPECT_EQ(actual, expected);
}
