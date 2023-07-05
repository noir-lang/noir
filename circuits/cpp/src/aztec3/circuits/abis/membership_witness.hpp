#pragma once

#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using std::is_same;

template <typename NCT, unsigned int N> struct MembershipWitness {
    using fr = typename NCT::fr;
    using boolean = typename NCT::boolean;

    fr leaf_index;
    std::array<fr, N> sibling_path{};

    MSGPACK_FIELDS(leaf_index, sibling_path);
    boolean operator==(MembershipWitness<NCT, N> const& other) const
    {
        return leaf_index == other.leaf_index && sibling_path == other.sibling_path;
    };

    template <typename Builder> MembershipWitness<CircuitTypes<Builder>, N> to_circuit_type(Builder& builder) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the circuit builder:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(builder, e); };

        MembershipWitness<CircuitTypes<Builder>, N> witness = {
            to_ct(leaf_index),
            map(sibling_path, to_ct),
        };

        return witness;
    }
};

template <typename NCT, unsigned int N> void read(uint8_t const*& it, MembershipWitness<NCT, N>& obj)
{
    using serialize::read;

    read(it, obj.leaf_index);
    read(it, obj.sibling_path);
};

template <typename NCT, unsigned int N> void write(std::vector<uint8_t>& buf, MembershipWitness<NCT, N> const& obj)
{
    using serialize::write;

    write(buf, obj.leaf_index);
    write(buf, obj.sibling_path);
};

template <typename NCT, unsigned int N> std::ostream& operator<<(std::ostream& os, MembershipWitness<NCT, N> const& obj)
{
    return os << "leaf_index: " << obj.leaf_index << "\n"
              << "sibling_path: " << obj.sibling_path << "\n";
}

}  // namespace aztec3::circuits::abis