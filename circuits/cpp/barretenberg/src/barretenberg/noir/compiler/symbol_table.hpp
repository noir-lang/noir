#pragma once
#include "var_t.hpp"
#include <map>

namespace noir {
namespace code_gen {

class SymbolTable {
    typedef std::map<std::string const, var_t> ScopeMap;

  public:
    SymbolTable();

    void set(var_t const& var, std::string const& key);

    void declare(var_t const& var, std::string const& key);

    var_t operator[](std::string const& key);

    void push();

    void pop();

  private:
    std::optional<ScopeMap::iterator> lookup(std::string const& key);

  private:
    std::vector<ScopeMap> variables_;
};

} // namespace code_gen
} // namespace noir