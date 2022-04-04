#pragma once
#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/convert.hpp>
// #include "contract_factory.hpp" // TODO: circular dependency
#include "l1_promise.hpp"
#include "l1_result.hpp"

namespace aztec3::circuits::apps {

using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

template <typename Composer> class ContractFactory;

// We use the struct to retain designated initialisation, to make contract creation more readable.
template <typename Composer> struct L1FunctionStruct {
    typedef typename CircuitTypes<Composer>::fr fr;

    std::string function_name;
    fr function_selector;
    size_t num_params = 0;
};

template <typename Composer> class L1Function {
    typedef typename CircuitTypes<Composer>::fr fr;

  public:
    ContractFactory<Composer>* contract_factory;
    std::string function_name;
    fr function_selector;
    size_t num_params;

    L1Function(){};

    L1Function(ContractFactory<Composer>* contract_factory, L1FunctionStruct<Composer> const& l1_fn_struct)
        : contract_factory(contract_factory)
        , function_name(l1_fn_struct.function_name)
        , function_selector(l1_fn_struct.function_selector)
        , num_params(l1_fn_struct.num_params)
    {}

    // L1Function(L1Function<Composer> const& l1_function)
    //     : contract_factory(l1_function.contract_factory)
    //     , function_name(l1_function.function_name)
    //     , function_selector(l1_function.function_selector)
    //     , num_params(l1_function.num_params)
    // {}

    std::pair<L1Promise<Composer>, L1Result> call(std::vector<fr> args);
};

} // namespace aztec3::circuits::apps

#include "l1_function.tpp"