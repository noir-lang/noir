#pragma once
#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/convert.hpp>
#include "l1_promise.hpp"
#include "l1_result.hpp"

namespace aztec3::circuits::apps {

using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

template <typename Composer> class Contract;

// We use the struct to retain designated initialisation, to make contract creation more readable.
template <typename Composer> struct L1FunctionInterfaceStruct {
    typedef typename CircuitTypes<Composer>::fr fr;

    std::string function_name;
    fr function_selector;
    size_t num_params = 0;
};

template <typename Composer> class L1FunctionInterface {
    typedef typename CircuitTypes<Composer>::fr fr;

  public:
    Contract<Composer>* contract;
    std::string function_name;
    fr function_selector;
    size_t num_params;

    L1FunctionInterface(){};

    L1FunctionInterface(Contract<Composer>* contract, L1FunctionInterfaceStruct<Composer> const& l1_fn_struct)
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

    std::pair<L1Promise<Composer>, L1Result> call(std::vector<fr> args)
    {
        if (args.size() != num_params) {
            throw_or_abort("Incorrect number of args");
        }

        auto promise = L1Promise<Composer>(*contract);
        L1Result result;
        return std::make_pair(promise, result);
    }
};

} // namespace aztec3::circuits::apps

// #include "l1_function_interface.tpp"