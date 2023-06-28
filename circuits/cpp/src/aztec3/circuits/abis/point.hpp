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

    Coordinate<NCT> x;
    Coordinate<NCT> y;

    // for serialization, update with new fields
    MSGPACK_FIELDS(x, y);
    bool operator==(Point<NCT> const&) const = default;

    template <typename Builder> Point<CircuitTypes<Builder>> to_circuit_type(Builder& builder) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        Point<CircuitTypes<Builder>> point = {
            x.to_circuit_type(builder),
            y.to_circuit_type(builder),
        };

        return point;
    };

    template <typename Builder> Point<NativeTypes> to_native_type() const
    {
        static_assert((std::is_same<CircuitTypes<Builder>, NCT>::value));

        auto to_native_type = []<typename T>(T& e) { return e.template to_native_type<Builder>(); };

        Point<NativeTypes> point = { to_native_type(x), to_native_type(y) };

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

template <typename NCT> void read(uint8_t const*& it, Point<NCT>& point)
{
    using serialize::read;

    read(it, point.x);
    read(it, point.y);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, Point<NCT> const& point)
{
    using serialize::write;

    write(buf, point.x);
    write(buf, point.y);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, Point<NCT> const& point)
{
    return os << "x: " << point.x << "\n"
              << "y: " << point.y << "\n";
}

}  // namespace aztec3::circuits::abis