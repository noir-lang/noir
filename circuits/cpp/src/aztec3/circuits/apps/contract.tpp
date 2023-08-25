#pragma once

#include "function_declaration.hpp"
#include "function_execution_context.hpp"

#include "aztec3/circuits/abis/function_data.hpp"
#include "aztec3/constants.hpp"
#include "aztec3/utils/types/convert.hpp"

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::apps {

using NT = aztec3::utils::types::NativeTypes;
using aztec3::circuits::abis::FunctionData;
using aztec3::circuits::abis::FunctionSelector;

template <typename NCT> void Contract<NCT>::set_functions(std::vector<FunctionDeclaration<NCT>> const& functions)
{
    for (uint32_t i = 0; i < functions.size(); ++i) {
        const auto& function = functions[i];
        if (function_datas.contains(function.name)) {
            throw_or_abort("Name already exists");
        }
        function_datas[function.name] = FunctionData<NCT>{
            .selector =
                {
                    .value = static_cast<uint32>(i),
                },
            .is_private = function.is_private,
            .is_constructor = function.is_constructor,
        };
    }
};

template <typename NCT>
void Contract<NCT>::import_contracts(std::vector<std::pair<std::string, Contract<NCT>>> const import_declarations)
{
    // Prevents an infinite loop if two contracts import each-other.
    if (already_imported) {
        return;
    }

    for (uint32_t i = 0; i < import_declarations.size(); ++i) {
        const std::pair<std::string, Contract<NCT>>& decl = import_declarations[i];
        if (imported_contracts.contains(decl.first)) {
            throw_or_abort("Name already exists");
        }
        imported_contracts.insert(decl);
    }
    already_imported = true;
}

// TODO: return some Function class which has a `call` method...
// FunctionData<CT> get_function(std::string name) { return function_data[name]; }

template <typename NCT> FunctionData<NCT> Contract<NCT>::get_function_data_by_name(std::string const& name)
{
    if (!function_datas.contains(name)) {
        throw_or_abort("function data not found");
    }
    return function_datas[name];
}

}  // namespace aztec3::circuits::apps