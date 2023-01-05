#pragma once

#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>

namespace aztec3::circuits::apps {

using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

/**
 * @tparam NCT - NativeTypes or CircuitTypes<Composer>
 * @tparam NotePreimage
 */
template <typename NCT, typename NotePreimage> struct UTXOSLoadDatum {
    typedef typename NCT::fr fr;
    typedef typename NCT::address address;
    typedef typename NCT::uint32 uint32;

    fr commitment;
    address contract_address;
    NotePreimage preimage;

    std::vector<fr> sibling_path;
    uint32 leaf_index;
    fr old_private_data_tree_root;

    template <typename Composer> auto to_circuit_type(Composer& composer) const
    {
        static_assert(std::is_same<NativeTypes, NCT>::value);

        // Capture the composer:
        auto to_ct = [&](auto& e) { return plonk::stdlib::types::to_ct(composer, e); };

        auto preimage_ct = preimage.to_circuit_type(composer);

        UTXOSLoadDatum<CircuitTypes<Composer>, decltype(preimage_ct)> datum = {
            to_ct(commitment),   to_ct(contract_address), preimage_ct,
            to_ct(sibling_path), to_ct(leaf_index),       to_ct(old_private_data_tree_root),
        };

        return datum;
    };
};

} // namespace aztec3::circuits::apps
