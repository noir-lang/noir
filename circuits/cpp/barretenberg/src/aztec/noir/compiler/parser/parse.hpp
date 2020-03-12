#pragma once
#include "ast.hpp"
#include "config.hpp"
#include "expression.hpp"

namespace noir {
namespace parser {

// GCC will not initialize the globals in the library in the right order (or at all?),
// unless the compiler can determine there is a reference to them somewhere. It all gets
// a bit hazy and non-standard at that point, but the only portable way appears to be
// ensuring that a compilation unit that depends on the libs globals references at least
// one of them. The block below ensures the globals in expression.o get initialized first.
extern boost::spirit::x3::symbols<ast::optoken> logical_op;
namespace {
auto forces_expr_global_init = []() { return &noir::parser::logical_op; }();
} // namespace

ast::statement_list parse(iterator_type begin, iterator_type end);
ast::statement_list parse(std::string const& source);
ast::function_statement_list parse_function_statements(std::string const&);

} // namespace parser
} // namespace noir