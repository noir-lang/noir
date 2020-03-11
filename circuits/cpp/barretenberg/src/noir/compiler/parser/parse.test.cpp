#include <fstream>
#include <gtest/gtest.h>
#include "parse.hpp"

using namespace noir::parser;

// namespace boost {
// void throw_exception(std::exception const&)
// {
//     std::abort();
// }
// } // namespace boost

TEST(noir, uint_sizes)
{
    auto ast = parse("          \n\
        uint2 my_int2 = 0;      \n\
        uint3 my_int3 = 0;      \n\
        uint32 my_int32 = 0;    \n\
        uint64 my_int64 = 0;    \n\
    ");

    auto type_id = boost::get<noir::ast::variable_declaration>(ast[0]).type;
    auto int_type = boost::get<noir::ast::int_type>(type_id.type);
    EXPECT_EQ(int_type.size, 2UL);
}

#ifndef __wasm__
TEST(noir, parse_fails)
{
    EXPECT_THROW(parse("1 + 2; blah"), std::runtime_error);
}

TEST(noir, uint1_fail)
{
    EXPECT_THROW(parse("uint1 my_int1 = 0;"), std::runtime_error);
}

TEST(noir, uint65_fail)
{
    EXPECT_THROW(parse("uint65 my_int65 = 0;"), std::runtime_error);
}
#endif

TEST(noir, function_definition)
{
    parse("uint32 my_function(uint32 arg1, bool arg2) {}");
}

TEST(noir, function_call)
{
    parse("bool x = my_function(arg1, 3+5+(x));");
}

TEST(noir, array_variable_definition)
{
    parse("uint32[4] my_var = [0x1, 0x12, 0x123, 0x1234];");
}

TEST(noir, array_expressions)
{
    parse_function_statements("uint32[4] my_var = [func_call(), 13, true];");
}

TEST(noir, array_index)
{
    parse_function_statements("my_var = some_array[5*3][1+2];");
}

TEST(noir, unary)
{
    parse_function_statements("my_var = !x;");
}
