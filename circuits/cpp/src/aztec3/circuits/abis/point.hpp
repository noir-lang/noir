#pragma once
#include "aztec3/circuits/abis/coordinate.hpp"
#include "aztec3/constants.hpp"
#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::abis {

using aztec3::GeneratorIndex;
using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;

template <typename NCT> struct Point {
    using fr = typename NCT::fr;
    using boolean = typename NCT::boolean;

    fr x;
    fr y;

    // for serialization, update with new fields
    MSGPACK_FIELDS(x, y);
    bool operator==(Point<NCT> const&) const = default;

    template <typename Builder> Point<CircuitTypes<Builder>> to_circuit_type(Builder& builder) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(builder, e); };

        Point<CircuitTypes<Builder>> point = { to_ct(x), to_ct(y) };

        return point;
    };

    template <typename Builder> Point<NativeTypes> to_native_type() const
    {
        static_assert((std::is_same<CircuitTypes<Builder>, NCT>::value));

        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Builder>(e); };

        Point<NativeTypes> point = { to_nt(x), to_nt(y) };

        return point;
    };

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        x.set_public();
        y.set_public();
    }

    void assert_is_zero()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        x.assert_is_zero();
        y.assert_is_zero();
    }
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, Point<NCT> const& point)
{
    return os << "x: " << point.x << "\n"
              << "y: " << point.y << "\n";
}

}  // namespace aztec3::circuits::abis
