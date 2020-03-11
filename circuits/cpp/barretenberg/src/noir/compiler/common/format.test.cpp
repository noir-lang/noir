#include <fstream>
#include <gtest/gtest.h>
#include "format.hpp"

// namespace boost {
// void throw_exception(std::exception const&)
// {
//     std::abort();
// }
// } // namespace boost

TEST(noir, format_string)
{
    EXPECT_EQ(noir::format("hello %s %d", "world", 123), "hello world 123");
}