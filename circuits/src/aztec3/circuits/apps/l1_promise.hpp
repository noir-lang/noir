#pragma once
#include <variant>
#include <crypto/pedersen/generator_data.hpp>
#include <stdlib/hash/pedersen/pedersen.hpp>
#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/convert.hpp>
#include <aztec3/circuits/abis/callback_stack_item.hpp>
// #include "contract_factory.hpp" // TODO: circular dependency?
#include "l1_result.hpp"

namespace aztec3::circuits::apps {

using aztec3::circuits::abis::CallbackStackItem;
using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

template <typename Composer> class ContractFactory;

template <typename Composer> class L1Promise {
    typedef CircuitTypes<Composer> CT;
    typedef typename CT::fr fr;
    typedef typename CT::grumpkin_point grumpkin_point;
    typedef typename CT::boolean boolean;

  public:
    ContractFactory<Composer>& contract_factory;
    CallbackStackItem<CT> callback_stack_item;

    L1Promise(ContractFactory<Composer>& contract_factory)
        : contract_factory(contract_factory)
    {}

    void on_success(std::string const& function_name, std::vector<std::variant<fr, size_t>> const& args);
    void on_failure(std::string const& function_name, std::vector<fr> const& args);
};

} // namespace aztec3::circuits::apps

#include "l1_promise.tpp"