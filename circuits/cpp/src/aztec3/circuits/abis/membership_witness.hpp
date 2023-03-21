#pragma once

#include "aztec3/utils/types/circuit_types.hpp"
namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using std::is_same;

template <typename NCT, unsigned int N> struct MembershipWitness {

    typedef typename NCT::fr fr;
    typedef typename NCT::uint32 uint32;

    uint32 leaf_index;
    std::array<fr, N> sibling_path;

    bool operator==(MembershipWitness<NCT, N> const&) const = default;

    static MembershipWitness<NCT, N> empty() { return { 0, std::array<fr, N>(N) }; };
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

} // namespace aztec3::circuits::abis