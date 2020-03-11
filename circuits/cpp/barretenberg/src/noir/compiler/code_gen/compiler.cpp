#include "compiler.hpp"
#include "expression_visitor.hpp"
#include "function_call.hpp"
#include "function_statement_visitor.hpp"
#include "../common/log.hpp"
#include "type_info_from.hpp"
#include "var_t.hpp"
#include <boost/assert.hpp>
#include <boost/format.hpp>
#include <boost/variant/apply_visitor.hpp>
#include <iostream>
#include <set>

namespace noir {
namespace code_gen {

Compiler::Compiler(Composer& composer)
    : ctx_(composer)
{
    load_builtins(ctx_);
}

void Compiler::operator()(ast::variable_declaration const& x)
{
    auto ti = type_info_from_type_id(ctx_, x.type);
    debug("global variable declaration %1% %2%", ti, x.variable);

    var_t v = var_t_factory(ti, ctx_.composer);
    debug("%1% initialized to: %2%", x.variable, v);

    ctx_.symbol_table.declare(v, x.variable);

    if (!x.assignment.has_value()) {
        abort("Global variables must be defined.");
    }

    ast::assignment assign;
    assign.lhs = x.variable;
    assign.rhs = x.assignment.value();
    ExpressionVisitor(ctx_, v.type)(assign);
}

void Compiler::operator()(ast::function_declaration const& x)
{
    debug("function declaration: %1%", x.name);
    ctx_.functions[x.name] = x;
}

void Compiler::operator()(ast::statement const& x)
{
    debug("statement");
    boost::apply_visitor(*this, x);
}

void Compiler::operator()(ast::statement_list const& x)
{
    for (auto const& s : x) {
        (*this)(s);
    }
}

std::pair<var_t, waffle::TurboProver> Compiler::start(ast::statement_list const& x, std::vector<var_t> const& args)
{
    // Parse top level statements, after which we can reference "main" function.
    (*this)(x);

    var_t result = function_call(ctx_, "main", args);

    if (ctx_.composer.get_num_gates()) {
        auto prover = ctx_.composer.create_prover();
        return std::make_pair(std::move(result), std::move(prover));
    } else {
        return std::make_pair(std::move(result), waffle::TurboProver());
    }
}

} // namespace code_gen
} // namespace noir