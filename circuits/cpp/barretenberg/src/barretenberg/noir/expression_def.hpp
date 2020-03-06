#pragma once
#include "ast.hpp"
#include "ast_adapted.hpp"
#include "common.hpp"
#include "expression.hpp"
#include <boost/spirit/home/x3.hpp>
#include <boost/spirit/home/x3/support/utility/annotate_on_success.hpp>

namespace noir {
namespace parser {

using x3::bool_;
using x3::char_;
using x3::hex;
using x3::lexeme;
using x3::raw;
using x3::uint_;
using namespace x3::ascii;

x3::symbols<ast::optoken> equality_op;
x3::symbols<ast::optoken> relational_op;
x3::symbols<ast::optoken> logical_op;
x3::symbols<ast::optoken> bitwise_op;
x3::symbols<ast::optoken> bitwise_shift_op;
x3::symbols<ast::optoken> additive_op;
x3::symbols<ast::optoken> multiplicative_op;
x3::symbols<ast::optoken> unary_op;
x3::symbols<ast::optoken> index_op;
x3::symbols<> keywords;

void add_keywords()
{
    // clang-format off
    static bool once = false;
    if (once)
        return;
    once = true;

    logical_op.add
        ("&&", ast::op_and)
        ("||", ast::op_or)
        ;

    bitwise_op.add
        ("^", ast::op_bitwise_xor)
        ("|", ast::op_bitwise_or)
        ("&", ast::op_bitwise_and)
        ;

    bitwise_shift_op.add
        (">>>", ast::op_bitwise_ror)
        ("<<<", ast::op_bitwise_rol)
        (">>", ast::op_bitwise_shr)
        ("<<", ast::op_bitwise_shl)
        ;

    equality_op.add
        ("==", ast::op_equal)
        ("!=", ast::op_not_equal)
        ;

    relational_op.add
        ("<", ast::op_less)
        ("<=", ast::op_less_equal)
        (">", ast::op_greater)
        (">=", ast::op_greater_equal)
        ;

    additive_op.add
        ("+", ast::op_plus)
        ("-", ast::op_minus)
        ;

    multiplicative_op.add
        ("*", ast::op_times)
        ("/", ast::op_divide)
        ("%", ast::op_mod)
        ;

    unary_op.add
        ("+", ast::op_positive)
        ("-", ast::op_negative)
        ("!", ast::op_not)
        ("~", ast::op_bitwise_not)
        ;

    index_op.add
        ("[", ast::op_index)
        ;

    keywords.add
        ("bool")
        ("true")
        ("false")
        ("for")
        ("return")
        ("mutable")
        ;
    // clang-format on
}

////////////////////////////////////////////////////////////////////////////
// Main expression grammar
////////////////////////////////////////////////////////////////////////////

typedef x3::rule<struct equality_expr_class, ast::expression> equality_expr_type;
typedef x3::rule<struct relational_expr_class, ast::expression> relational_expr_type;
typedef x3::rule<struct logical_expr_class, ast::expression> logical_expr_type;
typedef x3::rule<struct bitwise_expr_class, ast::expression> bitwise_expr_type;
typedef x3::rule<struct bitwise_shift_expr_class, ast::expression> bitwise_shift_expr_type;
typedef x3::rule<struct additive_expr_class, ast::expression> additive_expr_type;
typedef x3::rule<struct multiplicative_expr_class, ast::expression> multiplicative_expr_type;
typedef x3::rule<struct unary_expr_class, ast::operand> unary_expr_type;
typedef x3::rule<struct index_expr_class, ast::expression> index_expr_type;
typedef x3::rule<struct function_call_class, ast::function_call> function_call_type;
typedef x3::rule<struct primary_expr_class, ast::operand> primary_expr_type;
typedef x3::rule<struct constant_expr_class, ast::constant> constant_expr_type;
typedef x3::rule<struct array_expr_class, ast::array> array_expr_type;

expression_type const expression = "expression";
equality_expr_type const equality_expr = "equality_expr";
relational_expr_type const relational_expr = "relational_expr";
logical_expr_type const logical_expr = "logical_expr";
bitwise_expr_type const bitwise_expr = "bitwise_expr";
bitwise_shift_expr_type const bitwise_shift_expr = "bitwise_shift_expr";
additive_expr_type const additive_expr = "additive_expr";
multiplicative_expr_type const multiplicative_expr = "multiplicative_expr";
unary_expr_type const unary_expr = "unary_expr";
index_expr_type const index_expr = "index_expr";
function_call_type const function_call = "function_call";
primary_expr_type const primary_expr = "primary_expr";
constant_expr_type const constant_expr = "constant_expr";
array_expr_type const array_expr = "array_expr";

// clang-format off
auto const logical_expr_def =
        bitwise_expr
    >> *(logical_op > bitwise_expr)
    ;

auto const bitwise_expr_def =
        equality_expr
    >> *(bitwise_op > equality_expr)
    ;

auto const equality_expr_def =
        relational_expr
    >> *(equality_op > relational_expr)
    ;

auto const relational_expr_def =
        bitwise_shift_expr
    >> *(relational_op > bitwise_shift_expr)
    ;

auto const bitwise_shift_expr_def =
        additive_expr
    >> *(bitwise_shift_op > additive_expr)
    ;

auto const additive_expr_def =
        multiplicative_expr
    >> *(additive_op > multiplicative_expr)
    ;

auto const multiplicative_expr_def =
        unary_expr
    >> *(multiplicative_op > unary_expr)
    ;

auto const unary_expr_def =
        index_expr
    |   (unary_op > index_expr)
    ;

auto const index_expr_def =
        primary_expr
    >>  *(index_op > expression > "]")
    ;

auto const function_call_def =
        (!keywords >> identifier)
    >>  ("(" > -(expression % ',') > ")")
    ;

auto const constant_expr_def =
        ("0x" > hex)
    |   uint_
    |   bool_
    ;

auto const array_expr_def =
        "[" > (expression % ',') >  "]"
    ;

auto const primary_expr_def =
        constant_expr
    |   array_expr
    |   function_call
    |   (!keywords >> identifier)
    |   ('(' > expression > ')')
    ;

auto const expression_def = logical_expr;
// clang-format on

BOOST_SPIRIT_DEFINE(expression,
                    logical_expr,
                    bitwise_expr,
                    equality_expr,
                    relational_expr,
                    bitwise_shift_expr,
                    additive_expr,
                    multiplicative_expr,
                    unary_expr,
                    index_expr,
                    function_call,
                    constant_expr,
                    array_expr,
                    primary_expr);

struct unary_expr_class : x3::annotate_on_success {};
struct primary_expr_class : x3::annotate_on_success {};

} // namespace parser
} // namespace noir

namespace noir {
parser::expression_type const& expression()
{
    parser::add_keywords();
    return parser::expression;
}
} // namespace noir
