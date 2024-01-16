#include "barretenberg/serialize/cbind.hpp"
#include "barretenberg/serialize/msgpack.hpp"

#include <gtest/gtest.h>

// Sanity checking for msgpack
// TODO eventually move to barretenberg

struct GoodExample {
    bb::fr a;
    bb::fr b;
    MSGPACK_FIELDS(a, b);
} good_example;

struct BadExampleOverlap {
    bb::fr a;
    bb::fr b;
    MSGPACK_FIELDS(a, a);
} bad_example_overlap;

struct BadExampleIncomplete {
    bb::fr a;
    bb::fr b;
    MSGPACK_FIELDS(a);
} bad_example_incomplete;

struct BadExampleCompileTimeError {
    std::vector<int> a;
    bb::fr b;

    MSGPACK_FIELDS(b); // Type mismatch, expect 'a', will catch at compile-time
} bad_example_compile_time_error;

struct BadExampleOutOfObject {
    bb::fr a;
    bb::fr b;
    void msgpack(auto ar)
    {
        BadExampleOutOfObject other_object;
        ar("a", other_object.a, "b", other_object.b);
    }
} bad_example_out_of_object;

// TODO eventually move to barretenberg
TEST(msgpack_tests, msgpack_sanity_sanity)
{
    EXPECT_EQ(msgpack::check_msgpack_method(good_example), "");
    EXPECT_EQ(msgpack::check_msgpack_method(bad_example_overlap),
              "Overlap in BadExampleOverlap MSGPACK_FIELDS() params detected!");
    EXPECT_EQ(msgpack::check_msgpack_method(bad_example_incomplete),
              "Incomplete BadExampleIncomplete MSGPACK_FIELDS() params! Not all of object specified.");

    // If we actually try to msgpack BadExampleCompileTimeError we will statically error
    // This is great, but we need to check the underlying facility *somehow*
    auto checker = [&](auto&... values) {
        std::string incomplete_msgpack_status = "error";
        if constexpr (msgpack_concepts::MsgpackConstructible<BadExampleCompileTimeError, decltype(values)...>) {
            incomplete_msgpack_status = "";
        }
        EXPECT_EQ(incomplete_msgpack_status, "error");
    };
    bad_example_compile_time_error.msgpack(checker);

    EXPECT_EQ(msgpack::check_msgpack_method(bad_example_out_of_object),
              "Some BadExampleOutOfObject MSGPACK_FIELDS() params don't exist in object!");
}

struct ComplicatedSchema {
    std::vector<std::array<bb::fr, 20>> array;
    std::optional<GoodExample> good_or_not;
    bb::fr bare;
    std::variant<bb::fr, GoodExample> huh;
    MSGPACK_FIELDS(array, good_or_not, bare, huh);
} complicated_schema;

TEST(msgpack_tests, msgpack_schema_sanity)
{
    EXPECT_EQ(
        msgpack_schema_to_string(good_example),
        "{\"__typename\":\"GoodExample\",\"a\":[\"alias\",[\"fr\",\"bin32\"]],\"b\":[\"alias\",[\"fr\",\"bin32\"]]}\n");
    EXPECT_EQ(msgpack_schema_to_string(complicated_schema),
              "{\"__typename\":\"ComplicatedSchema\",\"array\":[\"vector\",[[\"array\",[[\"alias\",[\"fr\",\"bin32\"]],"
              "20]]]],\"good_or_not\":[\"optional\",[{\"__typename\":\"GoodExample\",\"a\":[\"alias\",[\"fr\","
              "\"bin32\"]],\"b\":[\"alias\",[\"fr\",\"bin32\"]]}]],\"bare\":[\"alias\",[\"fr\",\"bin32\"]],\"huh\":["
              "\"variant\",[[\"alias\",[\"fr\",\"bin32\"]],\"GoodExample\"]]}\n");
}
