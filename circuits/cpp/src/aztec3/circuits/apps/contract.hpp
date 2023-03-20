#pragma once

#include "function_declaration.hpp"
#include "l1_function_interface.hpp"

#include <aztec3/circuits/abis/function_data.hpp>

#include <barretenberg/common/container.hpp>

namespace aztec3::circuits::apps {

using aztec3::circuits::abis::FunctionData;

using aztec3::utils::types::CircuitTypes;
using plonk::stdlib::witness_t;
using NT = aztec3::utils::types::NativeTypes;

// template <typename Composer> class FunctionExecutionContext;

template <typename NCT> class Contract {
    typedef typename NCT::fr fr;
    typedef typename NCT::uint32 uint32;

  public:
    const std::string contract_name;

    fr state_var_counter = 0;

    std::vector<std::string> state_var_names;

    std::map<std::string, fr> start_slots_by_state_var_name;

    std::map<std::string, FunctionData<NCT>> function_datas;

    std::map<std::string, L1FunctionInterface<NCT>> l1_functions;

    std::map<std::string, Contract<NCT>> imported_contracts;

    Contract<NCT>(std::string const& contract_name)
        : contract_name(contract_name)
    {
        // exec_ctx.register_contract(this);
    }

    void set_functions(std::vector<FunctionDeclaration<NCT>> const& functions);

    void import_contracts(std::vector<std::pair<std::string, Contract<NCT>>> const import_declarations);

    Contract<NCT>& get_imported_contract(std::string const& name)
    {
        if (!imported_contracts.contains(name)) {
            throw_or_abort("No contract with that name imported");
        }
        return imported_contracts[name];
    }

    // TODO: return some Function class which has a `call` method...
    // FunctionData<CT> get_function(std::string name) { return function_data[name]; }

    FunctionData<NCT> get_function_data_by_name(std::string const& name);

    void import_l1_function(L1FunctionInterfaceStruct<NCT> const& l1_function_struct);

    L1FunctionInterface<NCT>& get_l1_function(std::string const& name);

    // TODO: maybe also declare a type at this stage, so the correct type can be checked-for when the StateVar type is
    // created within the function.
    /**
     * Note: this simply tracks the 'start' storage slots of each state variable at the 'contract scope level'.
     * TODO: maybe we can just keep a vector of names and query the start slot with index_of(), instead.
     */
    void declare_state_var(std::string const& state_var_name)
    {
        push_new_state_var_name(state_var_name);
        start_slots_by_state_var_name[state_var_name] = state_var_counter;
        // state_var_counter++;
        state_var_counter++;
        ASSERT(state_var_counter == state_var_names.size());
    };

    fr& get_start_slot(std::string const& state_var_name)
    {
        if (!start_slots_by_state_var_name.contains(state_var_name)) {
            throw_or_abort("Name '" + state_var_name + "' not found. Use `declare_state_var`.");
        }
        return start_slots_by_state_var_name.at(state_var_name);
    };

  private:
    // Prevents an infinite loop if two contracts import each other.
    bool already_imported;

    void push_new_state_var_name(std::string const& state_var_name)
    {
        if (index_of(state_var_names, state_var_name) == -1) {
            state_var_names.push_back(state_var_name);

            return;
        }
        throw_or_abort("name already exists");
    }
};

} // namespace aztec3::circuits::apps

// Importing in this way (rather than explicit instantiation of a template class at the bottom of a .cpp file) preserves
// the following:
// - We retain implicit instantiation of templates.
// - We don't implement method definitions in this file, to avoid a circular dependency with
// function_execution_context.hpp.
// TODO: things have changed since initially importing this .tpp file - maybe a conventional .cpp file is possible now
// instead...
#include "contract.tpp"