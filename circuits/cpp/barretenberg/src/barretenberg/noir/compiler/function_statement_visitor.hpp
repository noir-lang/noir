#pragma once
#include "compiler_context.hpp"

namespace noir {
namespace code_gen {

class FunctionStatementVisitor : boost::static_visitor<var_t> {
  public:
    FunctionStatementVisitor(CompilerContext& ctx, type_info const& target_type);

    var_t operator()(ast::variable_declaration const& x);
    var_t operator()(ast::expression const& x);
    var_t operator()(ast::assignment const& x);
    var_t operator()(ast::function_statement_list const& x);
    var_t operator()(ast::function_statement const& x);
    var_t operator()(boost::recursive_wrapper<ast::for_statement> const& x_);
    var_t operator()(ast::return_expr const& x);

  private:
    CompilerContext& ctx_;
    type_info const& target_type_;
    var_t return_;
};

} // namespace code_gen
} // namespace noir