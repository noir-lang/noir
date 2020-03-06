#pragma once
#include "ast.hpp"
#include <boost/fusion/include/adapt_struct.hpp>

BOOST_FUSION_ADAPT_STRUCT(noir::ast::variable, name, indexes)

BOOST_FUSION_ADAPT_STRUCT(noir::ast::unary, operator_, operand_)

BOOST_FUSION_ADAPT_STRUCT(noir::ast::operation, operator_, operand_)

BOOST_FUSION_ADAPT_STRUCT(noir::ast::expression, first, rest)

BOOST_FUSION_ADAPT_STRUCT(noir::ast::int_type, type, size)

BOOST_FUSION_ADAPT_STRUCT(noir::ast::type_id, qualifier, type, array_size)

BOOST_FUSION_ADAPT_STRUCT(noir::ast::variable_declaration, type, variable, assignment)

BOOST_FUSION_ADAPT_STRUCT(noir::ast::function_type_id, type, is_array, array_size)

BOOST_FUSION_ADAPT_STRUCT(noir::ast::function_argument, type, name)

BOOST_FUSION_ADAPT_STRUCT(noir::ast::function_declaration, return_type, name, args, statements)

BOOST_FUSION_ADAPT_STRUCT(noir::ast::function_call, name, args)

BOOST_FUSION_ADAPT_STRUCT(noir::ast::assignment, lhs, rhs)

BOOST_FUSION_ADAPT_STRUCT(noir::ast::for_statement, counter, from, to, body)

BOOST_FUSION_ADAPT_STRUCT(noir::ast::return_expr, expr)
