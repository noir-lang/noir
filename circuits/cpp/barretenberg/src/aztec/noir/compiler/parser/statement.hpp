#pragma once
#include "ast.hpp"
#include <boost/spirit/home/x3.hpp>

namespace noir {
namespace x3 = boost::spirit::x3;
namespace parser {

typedef x3::rule<struct statement_class, ast::statement_list> statement_type;
typedef x3::rule<struct function_statement_class, ast::function_statement_list> function_statement_type;

BOOST_SPIRIT_DECLARE(statement_type, function_statement_type);

} // namespace parser
parser::statement_type const& statement();
parser::function_statement_type const& function_statement();
} // namespace noir
