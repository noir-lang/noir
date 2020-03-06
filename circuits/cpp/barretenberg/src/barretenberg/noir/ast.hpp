#pragma once
#include <boost/fusion/include/io.hpp>
#include <boost/spirit/home/x3/support/ast/position_tagged.hpp>
#include <boost/spirit/home/x3/support/ast/variant.hpp>
#include <list>

namespace noir {
namespace ast {
namespace x3 = boost::spirit::x3;

struct nil {};
struct operation;
struct unary;
struct function_call;
struct expression;
typedef std::string bool_type;

struct variable : x3::position_tagged {
    variable(std::string const& name = "")
        : name(name)
    {}
    std::string name;
    std::vector<ast::expression> indexes;
};

struct constant : x3::variant<unsigned int, bool> {
    using base_type::base_type;
    using base_type::operator=;
};

struct array : std::vector<expression> {};

using operand = x3::variant<nil,
                            constant,
                            array,
                            variable,
                            x3::forward_ast<unary>,
                            x3::forward_ast<function_call>,
                            x3::forward_ast<expression>>;

enum optoken {
    op_plus,
    op_minus,
    op_times,
    op_divide,
    op_mod,
    op_positive,
    op_negative,
    op_not,
    op_equal,
    op_not_equal,
    op_less,
    op_less_equal,
    op_greater,
    op_greater_equal,
    op_and,
    op_or,
    op_bitwise_xor,
    op_bitwise_or,
    op_bitwise_and,
    op_bitwise_not,
    op_bitwise_shl,
    op_bitwise_shr,
    op_bitwise_rol,
    op_bitwise_ror,
    op_index,
};

enum qualifier_token {
    q_mutable,
};

struct operation : x3::position_tagged {
    optoken operator_;
    operand operand_;
};

struct expression : x3::position_tagged {
    operand first;
    std::list<operation> rest;
};

struct unary {
    optoken operator_;
    expression operand_;
};

struct assignment : x3::position_tagged {
    variable lhs;
    expression rhs;
};

struct function_call : x3::position_tagged {
    std::string name;
    std::vector<expression> args;
};

struct int_type {
    std::string type;
    size_t size;
};

struct type_id {
    std::optional<qualifier_token> qualifier;
    x3::variant<bool_type, int_type> type;
    std::optional<expression> array_size;
};

struct variable_declaration {
    type_id type;
    std::string variable;
    std::optional<expression> assignment;
};

struct statement_list;
struct function_statement_list;

struct function_type_id {
    x3::variant<bool_type, int_type> type;
    bool is_array;
    std::optional<unsigned int> array_size;
};

struct function_argument {
    function_type_id type;
    std::string name;
};

struct function_declaration {
    function_type_id return_type;
    std::string name;
    std::vector<function_argument> args;
    boost::recursive_wrapper<function_statement_list> statements;
};

struct for_statement;

struct return_expr {
    expression expr;
};

struct statement : x3::variant<function_declaration, variable_declaration, boost::recursive_wrapper<statement_list>> {
    using base_type::base_type;
    using base_type::operator=;
};

struct statement_list : std::vector<statement> {};

typedef x3::variant<variable_declaration, expression, assignment, boost::recursive_wrapper<for_statement>, return_expr>
    function_statement;

struct function_statement_list : std::vector<function_statement> {};

struct for_statement {
    std::string counter;
    expression from;
    expression to;
    function_statement_list body;
};

// print functions for debugging
inline std::ostream& operator<<(std::ostream& out, nil)
{
    out << "nil";
    return out;
}

inline std::ostream& operator<<(std::ostream& out, variable const& var)
{
    out << var.name;
    return out;
}
} // namespace ast
} // namespace noir
