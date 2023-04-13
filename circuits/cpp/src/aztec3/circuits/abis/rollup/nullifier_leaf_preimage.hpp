#pragma once

#include "aztec3/utils/types/circuit_types.hpp"
#include "barretenberg/stdlib/merkle_tree/hash.hpp"
#include "barretenberg/stdlib/merkle_tree/nullifier_tree/nullifier_leaf.hpp"
namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using std::is_same;

template <typename NCT> struct NullifierLeafPreimage {

    typedef typename NCT::fr fr;
    typedef typename NCT::uint32 uint32;

    fr leaf_value = 0;
    uint32 next_index;
    fr next_value = 0;

    bool operator==(NullifierLeafPreimage<NCT> const&) const = default;

    fr hash() const { return stdlib::merkle_tree::hash_multiple_native({ leaf_value, next_index, next_value }); }
};

template <typename NCT> void read(uint8_t const*& it, NullifierLeafPreimage<NCT>& obj)
{
    using serialize::read;

    read(it, obj.leaf_value);
    read(it, obj.next_value);
    read(it, obj.next_index);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, NullifierLeafPreimage<NCT> const& obj)
{
    using serialize::write;

    write(buf, obj.leaf_value);
    write(buf, obj.next_value);
    write(buf, obj.next_index);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, NullifierLeafPreimage<NCT> const& obj)
{
    return os << "leaf_value: " << obj.leaf_value << "\n"
              << "next_value: " << obj.next_value << "\n"
              << "next_index: " << obj.next_index << "\n";
}

} // namespace aztec3::circuits::abis