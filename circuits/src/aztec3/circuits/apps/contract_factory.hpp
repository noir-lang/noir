#pragma once
#include <common/container.hpp>
#include <aztec3/constants.hpp>
#include <stdlib/types/convert.hpp>
#include <aztec3/circuits/abis/function_signature.hpp>
#include "private_state_note.hpp"
#include "private_state_var.hpp"
#include "function.hpp"
#include "l1_function.hpp"
#include "oracle_wrapper.hpp"

namespace aztec3::circuits::apps {

using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using NT = plonk::stdlib::types::NativeTypes;
using aztec3::circuits::abis::FunctionSignature;

template <typename Composer> class ContractFactory {
    typedef CircuitTypes<Composer> CT;
    typedef typename CT::fr fr;
    typedef typename CT::uint32 uint32;

  public:
    Composer& composer;
    OracleWrapperInterface<Composer>& oracle;
    const std::string contract_name;
    PrivateStateFactory<Composer> private_state_factory;
    std::map<std::string, FunctionSignature<CT>> function_signatures;
    std::map<std::string, L1Function<Composer>> l1_functions;

    ContractFactory<Composer>(Composer& composer, OracleWrapperInterface<Composer>& oracle, std::string contract_name)
        : composer(composer)
        , oracle(oracle)
        , contract_name(contract_name)
        , private_state_factory(PrivateStateFactory<Composer>(composer, oracle, contract_name))
    {}

    void set_functions(std::vector<Function<CT>> const& functions)
    {
        for (uint32_t i = 0; i < functions.size(); ++i) {
            const auto& function = functions[i];
            if (function_signatures.contains(function.name)) {
                throw_or_abort("Name already exists");
            }
            function_signatures[function.name] = FunctionSignature<CT>{
                .contract_address = oracle.get_this_contract_address(),
                .vk_index = uint32(i),
                .is_private = function.is_private,
                .is_constructor = function.is_constructor,
            };
        }
    };

    FunctionSignature<CT> get_function_signature_by_name(std::string const& name)
    {
        if (!function_signatures.contains(name)) {
            throw_or_abort("function signature not found");
        }
        return function_signatures[name];
    }

    // FunctionSignature<CircuitTypes<Composer>> get_function_signature_by_name(std::string const& name) {

    // }

    void import_l1_function(L1FunctionStruct<Composer> const& l1_function_struct)
    {
        L1Function<Composer> l1_function = L1Function<Composer>(this, l1_function_struct);
        l1_functions.insert(std::make_pair(l1_function_struct.function_name, l1_function));
    };

    L1Function<Composer>& get_l1_function(std::string const& name)
    {
        if (!l1_functions.contains(name)) {
            throw_or_abort("L1 function not found. Make sure to import_l1_function()");
        }
        return l1_functions[name];
    }

    PrivateStateVar<Composer>& new_private_state(std::string const& name,
                                                 PrivateStateType const& private_state_type = PARTITIONED)
    {
        return private_state_factory.new_private_state(name, private_state_type);
    };

    // For initialising a private state which is a mapping.
    PrivateStateVar<Composer>& new_private_state(std::string const& name,
                                                 std::vector<std::string> const& mapping_key_names,
                                                 PrivateStateType const& private_state_type = PARTITIONED)
    {
        return private_state_factory.new_private_state(name, mapping_key_names, private_state_type);
    };

    void finalise() { private_state_factory.finalise(); };

    PrivateStateVar<Composer>& get_private_state(std::string const& name) { return private_state_factory.get(name); };
};

} // namespace aztec3::circuits::apps