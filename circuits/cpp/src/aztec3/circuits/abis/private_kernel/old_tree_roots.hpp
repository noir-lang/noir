#pragma once
#include <barretenberg/stdlib/primitives/witness/witness.hpp>
#include <aztec3/utils/types/native_types.hpp>
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/convert.hpp>

namespace aztec3::circuits::abis::private_kernel {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using plonk::stdlib::witness_t;
using std::is_same;

template <typename NCT> struct OldTreeRoots {
    typedef typename NCT::fr fr;

    fr private_data_tree_root;
    fr nullifier_tree_root;
    fr contract_tree_root;
    fr private_kernel_vk_tree_root; // TODO: future enhancement

    template <typename Composer> OldTreeRoots<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(composer, e); };

        OldTreeRoots<CircuitTypes<Composer>> data = {
            to_ct(private_data_tree_root),
            to_ct(nullifier_tree_root),
            to_ct(contract_tree_root),
            to_ct(private_kernel_vk_tree_root),
        };

        return data;
    };

    template <typename Composer> OldTreeRoots<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Composer>, NCT>::value);
        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Composer>(e); };

        OldTreeRoots<NativeTypes> data = {
            to_nt(private_data_tree_root),
            to_nt(nullifier_tree_root),
            to_nt(contract_tree_root),
            to_nt(private_kernel_vk_tree_root),
        };

        return data;
    };

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        private_data_tree_root.set_public();
        nullifier_tree_root.set_public();
        contract_tree_root.set_public();
        private_kernel_vk_tree_root.set_public();
    }
};

} // namespace aztec3::circuits::abis::private_kernel