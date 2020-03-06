#include "compiler_context.hpp"
#include "var_t.hpp"

namespace noir {
namespace code_gen {

void load_builtins(CompilerContext& ctx);

BuiltinFunction const builtin_lookup(CompilerContext& ctx, std::string const& function_name);

ast::function_declaration const& function_lookup(CompilerContext& ctx,
                                                 std::string const& function_name,
                                                 size_t num_args);

var_t function_call(CompilerContext& ctx, ast::function_declaration const& func, std::vector<var_t> const& args);

var_t function_call(CompilerContext& ctx, std::string const& func_name, std::vector<var_t> const& args);

} // namespace code_gen
} // namespace noir