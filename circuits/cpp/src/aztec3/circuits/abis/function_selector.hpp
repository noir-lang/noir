#pragma once

#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include "barretenberg/serialize/msgpack.hpp"

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;

template <typename NCT> struct FunctionSelector {
    using uint32 = typename NCT::uint32;
    using boolean = typename NCT::boolean;
    using fr = typename NCT::fr;

    uint32 value;  // e.g. 1st 4-bytes of abi-encoding of function.

    MSGPACK_FIELDS(value);

    boolean operator==(FunctionSelector<NCT> const& other) const { return value == other.value; };

    template <typename Builder> FunctionSelector<CircuitTypes<Builder>> to_circuit_type(Builder& builder) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the circuit builder:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(builder, e); };

        FunctionSelector<CircuitTypes<Builder>> function_selector = {
            to_ct(value),
        };

        return function_selector;
    };

    template <typename Builder> FunctionSelector<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Builder>, NCT>::value);
        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Builder>(e); };

        FunctionSelector<NativeTypes> function_selector = {
            to_nt(value),
        };

        return function_selector;
    };

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        fr(value).set_public();
    }
};

}  // namespace aztec3::circuits::abis