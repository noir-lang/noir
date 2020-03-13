#pragma once
#include "symbol_table.hpp"
#include <map>

namespace noir {
namespace code_gen {

typedef std::map<std::string const, ast::function_declaration> FunctionMap;
typedef std::function<var_t(std::vector<var_t> const&)> BuiltinFunction;
typedef std::map<std::string const, BuiltinFunction> BuiltinMap;

struct CompilerContext {
    CompilerContext(Composer& composer)
        : composer(composer)
    {}
    Composer& composer;
    SymbolTable symbol_table;
    FunctionMap functions;
    BuiltinMap builtins;
};

} // namespace code_gen
} // namespace noir