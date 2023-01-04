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

template <typename Composer>
void Contract<Composer>::set_functions(std::vector<FunctionDeclaration<CircuitTypes<Composer>>> const& functions)
{
    for (uint32_t i = 0; i < functions.size(); ++i) {
        const auto& function = functions[i];
        if (function_signatures.contains(function.name)) {
            throw_or_abort("Name already exists");
        }
        function_signatures[function.name] = FunctionSignature<CT>{
            // .contract_address = exec.oracle.get_this_contract_address(),
            .vk_index = uint32(i),
            .is_private = function.is_private,
            .is_constructor = function.is_constructor,
        };
    }
};

// TODO: return some Function class which has a `call` method...
// FunctionSignature<CT> get_function(std::string name) { return function_signature[name]; }

template <typename Composer>
FunctionSignature<CircuitTypes<Composer>> Contract<Composer>::get_function_signature_by_name(std::string const& name)
{
    if (!function_signatures.contains(name)) {
        throw_or_abort("function signature not found");
    }
    return function_signatures[name];
}

template <typename Composer>
void Contract<Composer>::import_l1_function(L1FunctionInterfaceStruct<Composer> const& l1_function_struct)
{
    L1FunctionInterface<Composer> l1_function = L1FunctionInterface<Composer>(this, l1_function_struct);
    l1_functions.insert(std::make_pair(l1_function_struct.function_name, l1_function));
};

template <typename Composer> L1FunctionInterface<Composer>& Contract<Composer>::get_l1_function(std::string const& name)
{
    if (!l1_functions.contains(name)) {
        throw_or_abort("L1 function not found. Make sure to import_l1_function()");
    }
    return l1_functions[name];
}

} // namespace aztec3::circuits::apps