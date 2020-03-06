#pragma once
#include <boost/spirit/home/x3.hpp>

namespace noir {
namespace parser {

namespace x3 = boost::spirit::x3;

// clang-format off
auto const space_comment =
        x3::ascii::space
    |   x3::lexeme["/*" > *(x3::char_ - "*/") > "*/"]
    |   x3::lexeme["//" >> *(x3::char_ - x3::eol) >> (x3::eol | x3::eoi)];
// clang-format on

} // namespace parser
} // namespace noir