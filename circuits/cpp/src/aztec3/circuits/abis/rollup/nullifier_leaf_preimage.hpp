#pragma once

#include "aztec3/utils/types/circuit_types.hpp"

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using std::is_same;

template <typename NCT> struct NullifierLeafPreimage {
    using fr = typename NCT::fr;
    using uint32 = typename NCT::uint32;

    fr leaf_value = 0;
    fr next_value = 0;
    uint32 next_index;

    MSGPACK_FIELDS(leaf_value, next_value, next_index);
    bool operator==(NullifierLeafPreimage<NCT> const&) const = default;

    bool is_empty() const { return leaf_value.is_zero() && next_index == 0 && next_value.is_zero(); }

    fr hash() const
    {
        return is_empty() ? fr::zero()
                          : stdlib::merkle_tree::hash_multiple_native({ leaf_value, next_index, next_value });
    }
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, NullifierLeafPreimage<NCT> const& obj)
{
    return os << "leaf_value: " << obj.leaf_value << "\n"
              << "next_value: " << obj.next_value << "\n"
              << "next_index: " << obj.next_index << "\n";
}

}  // namespace aztec3::circuits::abis
