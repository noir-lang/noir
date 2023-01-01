#pragma once

#include "function_declaration.hpp"
#include "private_state_var.hpp"
#include "l1_function_interface.hpp"

#include <aztec3/circuits/abis/function_signature.hpp>

namespace aztec3::circuits::apps {

using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using NT = plonk::stdlib::types::NativeTypes;
using aztec3::circuits::abis::FunctionSignature;

template <typename Composer> class FunctionExecutionContext;

template <typename Composer> class Contract {
    typedef CircuitTypes<Composer> CT;
    typedef typename CT::fr fr;
    typedef typename CT::uint32 uint32;

  public:
    FunctionExecutionContext<Composer>& exec_ctx;

    const std::string contract_name;

    fr state_counter = 0;

    std::vector<std::string> state_names;

    std::map<std::string, PrivateStateVar<Composer>> private_state_vars;

    std::map<std::string, FunctionSignature<CT>> function_signatures;

    std::map<std::string, L1FunctionInterface<Composer>> l1_functions;

    Contract<Composer>(FunctionExecutionContext<Composer>& exec_ctx, std::string const& contract_name)
        : exec_ctx(exec_ctx)
        , contract_name(contract_name)
    {
        exec_ctx.register_contract(this);
    }

    void set_functions(std::vector<FunctionDeclaration<CT>> const& functions);

    // TODO: return some Function class which has a `call` method...
    // FunctionSignature<CT> get_function(std::string name) { return function_signature[name]; }

    FunctionSignature<CT> get_function_signature_by_name(std::string const& name);

    void import_l1_function(L1FunctionInterfaceStruct<Composer> const& l1_function_struct);

    L1FunctionInterface<Composer>& get_l1_function(std::string const& name);

    void push_new_state_var_name(std::string const& name);

    PrivateStateVar<Composer>& declare_private_state_var(std::string const& name,
                                                         PrivateStateType const& private_state_type = PARTITIONED);

    // For initialising a private state which is a mapping.
    PrivateStateVar<Composer>& declare_private_state_var(std::string const& name,
                                                         std::vector<std::string> const& mapping_key_names,
                                                         PrivateStateType const& private_state_type = PARTITIONED);

    PrivateStateVar<Composer>& get_private_state_var(std::string const& name);
};

} // namespace aztec3::circuits::apps

// Importing in this way (rather than explicit instantiation of a template class at the bottom of a .cpp file) preserves
// the following:
// - We retain implicit instantiation of templates, meaning we can pick and choose (with static_assert) which class
// methods support native,
//   circuit or both types.
// - We don't implement method definitions in this file, to avoid a circular dependency with
// function_execution_context.hpp.
#include "contract.tpp"