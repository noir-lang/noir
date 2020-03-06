#include "symbol_table.hpp"
#include "log.hpp"
#include <iostream>

namespace noir {
namespace code_gen {

SymbolTable::SymbolTable()
{
    push();
}

void SymbolTable::set(var_t const& var, std::string const& key)
{
    auto existing = lookup(key);
    if (existing.has_value()) {
        var_t& e = (*existing.value()).second;
        auto e_type = e.type.type_name();
        auto v_type = var.type.type_name();
        if (e_type != v_type) {
            abort(format("Cannot assign value with type %s to variable %s with type %s.", v_type, key, e_type));
        }
        debug("SYMBOL TABLE UPDATE: %1%", key);
        e = var;
    } else {
        abort("Symbol not found: " + key);
    }
}

void SymbolTable::declare(var_t const& var, std::string const& key)
{
    if (variables_.back().find(key) != variables_.back().end()) {
        abort("Symbol already defined in current scope: " + key);
    }
    debug("SYMBOL TABLE ADD: %1%", key);
    variables_.back().insert(std::make_pair(key, var));
}

var_t SymbolTable::operator[](std::string const& key)
{
    auto it = lookup(key);
    if (it.has_value()) {
        return var_t_ref((*(it.value())).second);
    }
    abort("Symbol not found: " + key);
}

void SymbolTable::push()
{
    variables_.push_back(ScopeMap());
}

void SymbolTable::pop()
{
    variables_.pop_back();
}

std::optional<SymbolTable::ScopeMap::iterator> SymbolTable::lookup(std::string const& key)
{
    for (auto it = variables_.rbegin(); it != variables_.rend(); ++it) {
        auto var = (*it).find(key);
        if (var == (*it).end()) {
            continue;
        } else {
            return var;
        }
    }
    return {};
}

} // namespace code_gen
} // namespace noir