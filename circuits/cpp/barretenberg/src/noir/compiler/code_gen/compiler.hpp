#pragma once
#include <plonk/composer/turbo_composer.hpp>
#include "compiler_context.hpp"
#include "var_t.hpp"

namespace noir {
namespace code_gen {

class Compiler {
  public:
    typedef void result_type;

    Compiler(Composer& composer);

    std::pair<var_t, waffle::TurboProver> start(ast::statement_list const& x, std::vector<var_t> const& args);

    void operator()(ast::variable_declaration const& x);
    void operator()(ast::function_declaration const& x);
    void operator()(ast::statement const& x);
    void operator()(ast::statement_list const& x);

  private:
    CompilerContext ctx_;
};

} // namespace code_gen
} // namespace noir
