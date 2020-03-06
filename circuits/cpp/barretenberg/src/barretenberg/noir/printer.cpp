#include "printer.hpp"
#include <boost/assert.hpp>
#include <boost/variant/apply_visitor.hpp>
#include <iostream>
#include <set>

namespace noir {
namespace code_gen {

void printer::operator()(unsigned int x) const
{
    std::cout << x << std::endl;
}

void printer::operator()(bool x) const
{
    std::cout << x << std::endl;
}

void printer::operator()(ast::variable const& x) const
{
    std::cout << x.name << std::endl;
}

void printer::operator()(ast::operation const& x) const
{
    boost::apply_visitor(*this, x.operand_);

    switch (x.operator_) {
    case ast::op_plus:
        std::cout << "op_add" << std::endl;
        break;
    case ast::op_minus:
        std::cout << "op_sub" << std::endl;
        break;
    case ast::op_times:
        std::cout << "op_times" << std::endl;
        break;
    case ast::op_divide:
        std::cout << "op_divide" << std::endl;
        break;

    case ast::op_equal:
        std::cout << "op_equal" << std::endl;
        break;
    case ast::op_not_equal:
        std::cout << "op_ne" << std::endl;
        break;
    case ast::op_less:
        std::cout << "op_lt" << std::endl;
        break;
    case ast::op_less_equal:
        std::cout << "op_lte" << std::endl;
        break;
    case ast::op_greater:
        std::cout << "op_gt" << std::endl;
        break;
    case ast::op_greater_equal:
        std::cout << "op_gte" << std::endl;
        break;

    case ast::op_and:
        std::cout << "op_and" << std::endl;
        break;
    case ast::op_or:
        std::cout << "op_or" << std::endl;
        break;

    case ast::op_bitwise_and:
        std::cout << "op_bitwise_and" << std::endl;
        break;
    case ast::op_bitwise_or:
        std::cout << "op_bitwise_or" << std::endl;
        break;
    case ast::op_bitwise_xor:
        std::cout << "op_bitwise_xor" << std::endl;
        break;
    default:
        BOOST_ASSERT(0);
    }
}

void printer::operator()(ast::unary const& x) const
{
    (*this)(x.operand_);

    switch (x.operator_) {
    case ast::op_negative:
        std::cout << "op_neg" << std::endl;
        break;
    case ast::op_not:
        std::cout << "op_not" << std::endl;
        break;
    case ast::op_positive:
        break;
    default:
        BOOST_ASSERT(0);
    }
}

void printer::operator()(ast::expression const& x) const
{
    boost::apply_visitor(*this, x.first);
    for (ast::operation const& oper : x.rest) {
        (*this)(oper);
    }
}

void printer::operator()(ast::assignment const& x) const
{
    (*this)(x.rhs);
    std::cout << "op_store" << std::endl;
}

void printer::operator()(ast::variable_declaration const& x) const
{
    std::cout << "variable declaration " << x.variable << std::endl;
    if (x.assignment.has_value()) {
        (*this)(x.assignment.value());
    }
}

void printer::operator()(ast::function_declaration const& x) const
{
    std::cout << "function declaration: " << x.return_type.type << " " << x.name << std::endl;
}

void printer::operator()(ast::statement const& x) const
{
    std::cout << "statement" << std::endl;
    boost::apply_visitor(*this, x);
}

void printer::operator()(ast::statement_list const& x) const
{
    for (auto const& s : x) {
        (*this)(s);
    }
}

void printer::start(ast::statement_list const& x) const
{
    (*this)(x);
}

} // namespace code_gen
} // namespace noir