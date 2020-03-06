#pragma once
#include "ast.hpp"
#include <boost/spirit/home/x3.hpp>

namespace noir {
namespace x3 = boost::spirit::x3;
namespace parser {

struct expression_class;
typedef x3::rule<expression_class, ast::expression> expression_type;
BOOST_SPIRIT_DECLARE(expression_type);

} // namespace parser
parser::expression_type const& expression();
} // namespace noir
