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

template <typename NCT> struct PublicDataUpdateRequest {
    using fr = typename NCT::fr;
    using boolean = typename NCT::boolean;

    fr leaf_index = 0;
    fr old_value = 0;
    fr new_value = 0;

    // for serialization, update with new fields
    MSGPACK_FIELDS(leaf_index, old_value, new_value);
    bool operator==(PublicDataUpdateRequest<NCT> const&) const = default;

    template <typename Builder> PublicDataUpdateRequest<CircuitTypes<Builder>> to_circuit_type(Builder& builder) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the circuit builder:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(builder, e); };

        PublicDataUpdateRequest<CircuitTypes<Builder>> update_request = {
            to_ct(leaf_index),
            to_ct(old_value),
            to_ct(new_value),
        };

        return update_request;
    };

    template <typename Builder> PublicDataUpdateRequest<NativeTypes> to_native_type() const
    {
        static_assert((std::is_same<CircuitTypes<Builder>, NCT>::value));

        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Builder>(e); };

        PublicDataUpdateRequest<NativeTypes> update_request = {
            to_nt(leaf_index),
            to_nt(old_value),
            to_nt(new_value),
        };

        return update_request;
    };

    fr hash() const
    {
        std::vector<fr> inputs = {
            leaf_index,
            old_value,
            new_value,
        };

        return NCT::compress(inputs, GeneratorIndex::PUBLIC_DATA_UPDATE_REQUEST);
    }

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        leaf_index.set_public();
        old_value.set_public();
        new_value.set_public();
    }

    boolean is_empty() const { return leaf_index == 0; }
};

}  // namespace aztec3::circuits::abis
