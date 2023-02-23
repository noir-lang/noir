#pragma once
#include "public_inputs.hpp"
#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/convert.hpp>

namespace aztec3::circuits::abis::private_kernel {

using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;
using std::is_same;

template <typename NCT> struct PreviousKernelData {
    typedef typename NCT::fr fr;
    typedef typename NCT::VK VK;

    PublicInputs<NCT> public_inputs; // TODO: not needed as already contained in proof?
    NativeTypes::Proof proof;        // TODO: how to express proof as native/circuit type when it gets used as a buffer?
    std::shared_ptr<VK> vk;

    // TODO: this index and path are meant to be those of a leaf within the tree of _kernel circuit_ vks; not the tree
    // of functions within the contract tree.
    fr vk_index;
    std::array<fr, VK_TREE_HEIGHT> vk_path;

    // WARNING: the `proof` does NOT get converted!
    template <typename Composer> PreviousKernelData<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        typedef CircuitTypes<Composer> CT;
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return plonk::stdlib::types::to_ct(composer, e); };

        PreviousKernelData<CircuitTypes<Composer>> data = {
            public_inputs.to_circuit_type(composer),
            proof, // Notice: not converted! Stays as native.
            CT::VK::from_witness(&composer, vk),
            to_ct(vk_index),
            to_ct(vk_path),
        };

        return data;
    };
}; // namespace aztec3::circuits::abis::private_kernel

} // namespace aztec3::circuits::abis::private_kernel