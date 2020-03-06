#pragma once
#include <boost/spirit/home/x3.hpp>

namespace noir {
namespace parser {

using x3::alnum;
using x3::alpha;
using x3::lexeme;
using x3::raw;

struct identifier_class;
typedef x3::rule<identifier_class, std::string> identifier_type;
identifier_type const identifier = "identifier";

auto const identifier_def = raw[lexeme[(alpha | '_') >> *(alnum | '_')]];

BOOST_SPIRIT_DEFINE(identifier);

} // namespace parser
} // namespace noir
