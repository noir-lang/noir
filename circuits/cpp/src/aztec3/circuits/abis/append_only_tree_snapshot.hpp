#pragma once

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::abis {

template <typename NCT> struct AppendOnlyTreeSnapshot {
    using fr = typename NCT::fr;
    using uint32 = typename NCT::uint32;

    fr root = 0;
    uint32 next_available_leaf_index;
    MSGPACK_FIELDS(root, next_available_leaf_index);

    bool operator==(AppendOnlyTreeSnapshot<NCT> const&) const = default;
};

}  // namespace aztec3::circuits::abis
