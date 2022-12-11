#pragma once
#include "function_execution_context.hpp"
#include "private_state_var.hpp"
#include "function_declaration.hpp"
#include "l1_function_interface.hpp"
#include <common/container.hpp>
#include <aztec3/constants.hpp>
#include <stdlib/types/convert.hpp>
#include <aztec3/circuits/abis/function_signature.hpp>

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

template <typename Composer> void Contract<Composer>::push_new_state_name(std::string const& name)
{
    if (index_of(state_names, name) == -1) {
        state_names.push_back(name);
        return;
    }
    throw_or_abort("name already exists");
}

template <typename Composer>
PrivateStateVar<Composer>& Contract<Composer>::new_private_state(std::string const& name,
                                                                 PrivateStateType const& private_state_type)
{
    push_new_state_name(name);
    PrivateStateVar<Composer> private_state_var =
        PrivateStateVar<Composer>(&exec_ctx, private_state_type, name, state_counter++);
    private_state_vars.insert(std::make_pair(name, private_state_var));
    return private_state_vars[name];
};

// For initialising a private state which is a mapping.
template <typename Composer>
PrivateStateVar<Composer>& Contract<Composer>::new_private_state(std::string const& name,
                                                                 std::vector<std::string> const& mapping_key_names,
                                                                 PrivateStateType const& private_state_type)
{
    push_new_state_name(name);
    PrivateStateVar<Composer> private_state_var =
        PrivateStateVar<Composer>(&exec_ctx, private_state_type, name, state_counter++, mapping_key_names);
    private_state_vars.insert(std::make_pair(name, private_state_var));
    return private_state_vars[name];
};

template <typename Composer> PrivateStateVar<Composer>& Contract<Composer>::get_private_state(std::string const& name)
{
    if (!private_state_vars.contains(name)) {
        throw_or_abort("name not found");
    }
    return private_state_vars[name];
};

} // namespace aztec3::circuits::apps