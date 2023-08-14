#pragma once
#include "aztec3/constants.hpp"
#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::abis {

using aztec3::GeneratorIndex;
using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;

template <typename NCT> struct Coordinate {
    using fr = typename NCT::fr;
    using boolean = typename NCT::boolean;

    std::array<fr, 2> fields;

    // for serialization, update with new fields
    MSGPACK_FIELDS(fields);
    bool operator==(Coordinate<NCT> const&) const = default;

    template <typename Builder> Coordinate<CircuitTypes<Builder>> to_circuit_type(Builder& builder) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the builder:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(builder, e); };

        Coordinate<CircuitTypes<Builder>> coordinate = {
            to_ct(fields),
        };

        return coordinate;
    };

    template <typename Builder> Coordinate<NativeTypes> to_native_type() const
    {
        static_assert((std::is_same<CircuitTypes<Builder>, NCT>::value));

        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Builder>(e); };

        Coordinate<NativeTypes> coordinate = {
            to_nt(fields),
        };

        return coordinate;
    };

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        fields[0].set_public();
        fields[1].set_public();
    }

    void assert_is_zero()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        fields[0].assert_is_zero();
        fields[1].assert_is_zero();
    }
};

}  // namespace aztec3::circuits::abis
