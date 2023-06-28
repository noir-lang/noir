#pragma once

#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::apps {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using plonk::stdlib::witness_t;

/**
 * @tparam NCT - NativeTypes or CircuitTypes<Builder>
 * @tparam NotePreimage
 */
template <typename NCT, typename NotePreimage> struct UTXOSLoadDatum {
    using fr = typename NCT::fr;
    using address = typename NCT::address;
    using uint32 = typename NCT::uint32;

    fr commitment = 0;
    address contract_address = 0;
    NotePreimage preimage{};

    std::vector<fr> sibling_path;
    uint32 leaf_index;
    fr historic_private_data_tree_root = 0;

    template <typename Builder> auto to_circuit_type(Builder& builder) const
    {
        static_assert(std::is_same<NativeTypes, NCT>::value);

        // Capture the circuit builder:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(builder, e); };

        auto preimage_ct = preimage.to_circuit_type(builder);

        UTXOSLoadDatum<CircuitTypes<Builder>, decltype(preimage_ct)> datum = {
            to_ct(commitment),   to_ct(contract_address), preimage_ct,
            to_ct(sibling_path), to_ct(leaf_index),       to_ct(historic_private_data_tree_root),
        };

        return datum;
    };
};

}  // namespace aztec3::circuits::apps
