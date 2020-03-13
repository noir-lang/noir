#pragma once
#include "ast.hpp"
#include "ast_adapted.hpp"
#include "common.hpp"
#include "error_handler.hpp"
#include "expression.hpp"
#include "statement.hpp"
#include "../common/log.hpp"
#include <boost/spirit/home/x3.hpp>
#include <boost/spirit/home/x3/support/utility/annotate_on_success.hpp>
#include <iostream>

namespace noir {
namespace parser {

using x3::char_;
using x3::lexeme;
using x3::raw;
using x3::string;
using x3::uint_;
using namespace x3::ascii;

x3::symbols<ast::qualifier_token> qualifier;

void add_symbols()
{
    qualifier.add("mutable", ast::q_mutable);
}

struct int_type_class {
    template <typename Iterator, typename Context>
    inline void on_success(Iterator const&, Iterator const&, ast::int_type& t, Context const&)
    {
        if (t.size < 2 || t.size > 64) {
            abort("Bad integer width.");
        }
    }
};

typedef x3::rule<struct statement_list_class, ast::statement_list> statement_list_type;
typedef x3::rule<int_type_class, ast::int_type> int_type_type;
typedef x3::rule<struct intrinsic_type_class, x3::variant<ast::bool_type, ast::int_type>> intrinsic_type_type;
typedef x3::rule<struct type_id_class, ast::type_id> type_id_type;
typedef x3::rule<struct variable_declaration_class, ast::variable_declaration> variable_declaration_type;
typedef x3::rule<struct function_type_id_class, ast::function_type_id> function_type_id_type;
typedef x3::rule<struct function_argument_class, ast::function_argument> function_argument_type;
typedef x3::rule<struct function_declaration_class, ast::function_declaration> function_declaration_type;
typedef x3::rule<struct function_statement_list_class, ast::function_statement_list> function_statement_list_type;
typedef x3::rule<struct for_statement_class, ast::for_statement> for_statement_type;
typedef x3::rule<struct assignment_class, ast::assignment> assignment_type;
typedef x3::rule<struct variable_class, ast::variable> variable_type;
typedef x3::rule<struct return_expr_class, ast::return_expr> return_expr_type;

statement_type const statement("statement");
statement_list_type const statement_list("statement_list");
intrinsic_type_type const intrinsic_type("intrinsic_type");
int_type_type const int_type("int_type");
type_id_type const type_id("type_id");
variable_declaration_type const variable_declaration("variable_declaration");
function_type_id_type const function_type_id("function_type_id");
function_argument_type const function_argument("function_argument");
function_declaration_type const function_declaration("function_declaration");
function_statement_type const function_statement("function_statement");
function_statement_list_type const function_statement_list("function_statement_list");
for_statement_type const for_statement("for_statement");
assignment_type const assignment("assignment");
variable_type const variable("variable");
return_expr_type const return_expr = "return_expr";

// Import the expression rule
namespace {
auto const& expression = noir::expression();
}

// clang-format off
auto const statement_list_def =
        *(function_declaration | variable_declaration)
    ;

auto const function_statement_list_def =
        *(variable_declaration | for_statement | return_expr | assignment | (expression > ';'))
    ;

auto const int_type_def =
        lexeme[string("uint") > uint_]
    |   lexeme[string("int") > uint_]
    ;

auto const intrinsic_type_def =
        lexeme[(string("bool") | int_type) >> !(alnum | '_')]
        ;

auto const type_id_def =
        -(qualifier)
    >>  intrinsic_type > -("[" > expression > "]")
    ;

auto const variable_declaration_def =
        type_id
    >   identifier > -("=" > expression)
    >   ";"
    ;

auto const function_type_id_def =
        intrinsic_type > -(char_('[') > -(uint_) > "]")
    ;

auto const function_argument_def =
        function_type_id > identifier
    ;

auto const function_declaration_def =
        (function_type_id > identifier)
    >>  ("(" > -(function_argument % ',') > ")")
    >>  ("{" > function_statement_list > "}")
    ;

auto const assignment_def =
        variable
    >>  ('='
    >   expression
    >   ';')
    ;

auto const for_statement_def =
        lit("for")
    >   "(" > identifier > "in" > expression > ".." > expression > ")"
    >   "{" > function_statement_list > "}"
    ;

auto const return_expr_def =
        lit("return")
    >   expression
    >   ";"
    ;

auto const variable_def =
        identifier
    >   *("[" > expression > "]")
    ;

auto const statement_def = statement_list;
auto const function_statement_def = function_statement_list;
// clang-format on

BOOST_SPIRIT_DEFINE(statement,
                    statement_list,
                    int_type,
                    intrinsic_type,
                    type_id,
                    variable_declaration,
                    function_type_id,
                    function_argument,
                    function_declaration,
                    function_statement,
                    function_statement_list,
                    assignment,
                    variable,
                    for_statement,
                    return_expr);

struct statement_class : error_handler_base, x3::annotate_on_success {};
struct assignment_class : x3::annotate_on_success {};
struct variable_class : x3::annotate_on_success {};
} // namespace parser

parser::statement_type const& statement()
{
    parser::add_symbols();
    return parser::statement;
}

parser::function_statement_type const& function_statement()
{
    parser::add_symbols();
    return parser::function_statement;
}
} // namespace noir
