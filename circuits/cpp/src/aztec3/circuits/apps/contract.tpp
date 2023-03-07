#pragma once
#include "function_execution_context.hpp"
#include "function_declaration.hpp"
#include "l1_function_interface.hpp"

#include <common/container.hpp>

#include <aztec3/constants.hpp>

#include <aztec3/circuits/abis/function_signature.hpp>

#include <stdlib/types/convert.hpp>

namespace aztec3::circuits::apps {

using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using NT = plonk::stdlib::types::NativeTypes;
using aztec3::circuits::abis::FunctionSignature;
// using aztec3::circuits::abis::OptionalPrivateCircuitPublicInputs;

template <typename NCT> void Contract<NCT>::set_functions(std::vector<FunctionDeclaration<NCT>> const& functions)
{
    for (uint32_t i = 0; i < functions.size(); ++i) {
        const auto& function = functions[i];
        if (function_signatures.contains(function.name)) {
            throw_or_abort("Name already exists");
        }
        function_signatures[function.name] = FunctionSignature<NCT>{
            .function_encoding = uint32(i),
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
// FunctionSignature<CT> get_function(std::string name) { return function_signature[name]; }

template <typename NCT> FunctionSignature<NCT> Contract<NCT>::get_function_signature_by_name(std::string const& name)
{
    if (!function_signatures.contains(name)) {
        throw_or_abort("function signature not found");
    }
    return function_signatures[name];
}

template <typename NCT> void Contract<NCT>::import_l1_function(L1FunctionInterfaceStruct<NCT> const& l1_function_struct)
{
    L1FunctionInterface<NCT> l1_function = L1FunctionInterface<NCT>(this, l1_function_struct);
    l1_functions.insert(std::make_pair(l1_function_struct.function_name, l1_function));
};

template <typename NCT> L1FunctionInterface<NCT>& Contract<NCT>::get_l1_function(std::string const& name)
{
    if (!l1_functions.contains(name)) {
        throw_or_abort("L1 function not found. Make sure to import_l1_function()");
    }
    return l1_functions[name];
}

} // namespace aztec3::circuits::apps