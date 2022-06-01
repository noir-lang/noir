#pragma once
#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/convert.hpp>

namespace aztec3::circuits::abis::private_kernel {

using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;
using std::is_same;

template <typename NCT> struct OldTreeRoots {
    typedef typename NCT::fr fr;

    fr private_data_tree_root;
    fr contract_tree_root;
    fr l1_results_tree_root;
    fr private_kernel_vk_tree_root; // TODO: future enhancement

    template <typename Composer> OldTreeRoots<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return plonk::stdlib::types::to_ct(composer, e); };

        OldTreeRoots<CircuitTypes<Composer>> data = {
            to_ct(private_data_tree_root),
            to_ct(contract_tree_root),
            to_ct(l1_results_tree_root),
            to_ct(private_kernel_vk_tree_root),
        };

        return data;
    };

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        private_data_tree_root.set_public();
        contract_tree_root.set_public();
        l1_results_tree_root.set_public();
        private_kernel_vk_tree_root.set_public();
    }
};

} // namespace aztec3::circuits::abis::private_kernel