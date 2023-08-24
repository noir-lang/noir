#pragma once

#include "aztec3/circuits/abis/function_selector.hpp"
#include "aztec3/constants.hpp"
#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "aztec3/utils/types/native_types.hpp"

namespace aztec3::circuits::abis {

// using plonk::stdlib::witness_t;
using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using std::is_same;

template <typename NCT> struct FunctionData {
    using uint32 = typename NCT::uint32;
    using boolean = typename NCT::boolean;
    using fr = typename NCT::fr;

    FunctionSelector<NCT> function_selector;
    boolean is_internal = false;
    boolean is_private = false;
    boolean is_constructor = false;

    MSGPACK_FIELDS(function_selector, is_internal, is_private, is_constructor);

    boolean operator==(FunctionData<NCT> const& other) const
    {
        return function_selector == other.function_selector && is_internal == other.is_internal &&
               is_private == other.is_private && is_constructor == other.is_constructor;
    };

    template <typename Builder> FunctionData<CircuitTypes<Builder>> to_circuit_type(Builder& builder) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the circuit builder:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(builder, e); };

        FunctionData<CircuitTypes<Builder>> function_data = {
            function_selector.to_circuit_type(builder),
            to_ct(is_internal),
            to_ct(is_private),
            to_ct(is_constructor),
        };

        return function_data;
    };

    template <typename Builder> FunctionData<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Builder>, NCT>::value);
        auto to_native_type = []<typename T>(T& e) { return e.template to_native_type<Builder>(); };
        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Builder>(e); };

        FunctionData<NativeTypes> function_data = {
            to_native_type(function_selector),
            to_nt(is_internal),
            to_nt(is_private),
            to_nt(is_constructor),
        };

        return function_data;
    };

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        function_selector.set_public();
        fr(is_internal).set_public();
        fr(is_private).set_public();
        fr(is_constructor).set_public();
    }

    // TODO: this can all be packed into 1 field element, so this `hash` function should just return that field element.
    fr hash() const
    {
        std::vector<fr> const inputs = {
            fr(function_selector.value),
            fr(is_internal),
            fr(is_private),
            fr(is_constructor),
        };

        return NCT::hash(inputs, GeneratorIndex::FUNCTION_DATA);
    }
};

}  // namespace aztec3::circuits::abis
