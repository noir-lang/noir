#pragma once
#include "../common/log.hpp"
#include "../parser/ast.hpp"
#include "compiler_context.hpp"
#include "symbol_table.hpp"

namespace noir {
namespace code_gen {

class ExpressionVisitor : boost::static_visitor<var_t> {
  public:
    ExpressionVisitor(CompilerContext& context, type_info const& target_type);

    var_t operator()(ast::nil) const
    {
        abort("nil");
        return var_t(false);
    }
    var_t operator()(unsigned int x);
    var_t operator()(bool x);
    var_t operator()(ast::assignment const& x);
    var_t operator()(ast::array const& x);
    var_t operator()(ast::variable const& x);
    var_t operator()(ast::function_call const& x);
    var_t operator()(var_t lhs, ast::operation const& x);
    var_t operator()(ast::unary const& x);
    var_t operator()(ast::expression const& x);
    var_t operator()(ast::constant const& x);

  private:
    var_t& get_symbol_table_var_ref(ast::variable const& x);

  private:
    CompilerContext& ctx_;
    type_info const& target_type_;
};

} // namespace code_gen
} // namespace noir