#pragma once

#include "aztec3/utils/array.hpp"
#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"

namespace aztec3::circuits::abis {

using aztec3::utils::is_array_empty;
using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using std::is_same;

template <typename NCT, unsigned int N> struct MembershipWitness {
    using fr = typename NCT::fr;
    using boolean = typename NCT::boolean;

    fr leaf_index = 0;
    std::array<fr, N> sibling_path{};

    MSGPACK_FIELDS(leaf_index, sibling_path);
    // for schema serialization
    void msgpack_schema(auto& packer) const { packer.pack_with_name("MembershipWitness" + std::to_string(N), *this); }
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

    template <typename Builder> MembershipWitness<NativeTypes, N> to_native_type() const
    {
        static_assert((std::is_same<CircuitTypes<Builder>, NCT>::value));

        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Builder>(e); };

        MembershipWitness<NativeTypes, N> witness = {
            to_nt(leaf_index),
            map(sibling_path, to_nt),
        };

        return witness;
    }

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        leaf_index.set_public();
        for (fr const& e : sibling_path) {
            e.set_public();
        }
    }

    boolean is_empty() const { return aztec3::utils::is_empty(leaf_index) && is_array_empty(sibling_path); }
};

}  // namespace aztec3::circuits::abis
