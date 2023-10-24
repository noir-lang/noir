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

template <typename NCT> struct PublicDataRead {
    using fr = typename NCT::fr;
    using boolean = typename NCT::boolean;

    fr leaf_index = 0;
    fr value = 0;

    // for serialization, update with new fields
    MSGPACK_FIELDS(leaf_index, value);
    bool operator==(PublicDataRead<NCT> const&) const = default;

    template <typename Builder> PublicDataRead<CircuitTypes<Builder>> to_circuit_type(Builder& builder) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the circuit builder:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(builder, e); };

        PublicDataRead<CircuitTypes<Builder>> read = {
            to_ct(leaf_index),
            to_ct(value),
        };

        return read;
    };

    template <typename Builder> PublicDataRead<NativeTypes> to_native_type() const
    {
        static_assert((std::is_same<CircuitTypes<Builder>, NCT>::value));

        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Builder>(e); };

        PublicDataRead<NativeTypes> read = {
            to_nt(leaf_index),
            to_nt(value),
        };

        return read;
    };

    fr hash() const
    {
        std::vector<fr> inputs = {
            leaf_index,
            value,
        };

        return NCT::hash(inputs, GeneratorIndex::PUBLIC_DATA_READ);
    }

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        leaf_index.set_public();
        value.set_public();
    }

    boolean is_empty() const { return leaf_index == 0; }
};

}  // namespace aztec3::circuits::abis
