#pragma once
#include <barretenberg/stdlib/primitives/witness/witness.hpp>
#include <aztec3/utils/types/native_types.hpp>
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/convert.hpp>

namespace aztec3::circuits::apps {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using plonk::stdlib::witness_t;

template <typename NCT> class Contract;

// We use the struct to retain designated initialisation, to make contract creation more readable.
template <typename NCT> struct L1FunctionInterfaceStruct {
    typedef typename NCT::fr fr;

    std::string function_name;
    fr function_selector;
    size_t num_params = 0;
};

template <typename NCT> class L1FunctionInterface {
    typedef typename NCT::fr fr;

  public:
    Contract<NCT>* contract;
    std::string function_name;
    fr function_selector;
    size_t num_params;

    L1FunctionInterface(){};

    L1FunctionInterface(Contract<NCT>* contract, L1FunctionInterfaceStruct<NCT> const& l1_fn_struct)
        : contract(contract)
        , function_name(l1_fn_struct.function_name)
        , function_selector(l1_fn_struct.function_selector)
        , num_params(l1_fn_struct.num_params)
    {}

    // L1FunctionInterface(L1FunctionInterface<Composer> const& l1_function)
    //     : contract(l1_function.contract)
    //     , function_name(l1_function.function_name)
    //     , function_selector(l1_function.function_selector)
    //     , num_params(l1_function.num_params)
    // {}

    void call(std::vector<fr> args)
    {
        // TODO: implement this function.
        (void)args; // So the compiler doesn't complain about an unused var.
    }
};

} // namespace aztec3::circuits::apps

// #include "l1_function_interface.tpp"