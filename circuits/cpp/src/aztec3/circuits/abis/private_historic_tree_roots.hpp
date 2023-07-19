#pragma once

#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using std::is_same;

template <typename NCT> struct PrivateHistoricTreeRoots {
    using fr = typename NCT::fr;
    using boolean = typename NCT::boolean;

    fr private_data_tree_root = 0;
    fr nullifier_tree_root = 0;
    fr contract_tree_root = 0;
    fr l1_to_l2_messages_tree_root = 0;
    fr private_kernel_vk_tree_root = 0;  // TODO: future enhancement

    // for serialization: update up with new fields
    MSGPACK_FIELDS(private_data_tree_root,
                   nullifier_tree_root,
                   contract_tree_root,
                   l1_to_l2_messages_tree_root,
                   private_kernel_vk_tree_root);

    boolean operator==(PrivateHistoricTreeRoots<NCT> const& other) const
    {
        return private_data_tree_root == other.private_data_tree_root &&
               nullifier_tree_root == other.nullifier_tree_root && contract_tree_root == other.contract_tree_root &&
               l1_to_l2_messages_tree_root == other.l1_to_l2_messages_tree_root &&
               private_kernel_vk_tree_root == other.private_kernel_vk_tree_root;
    };

    template <typename Builder> PrivateHistoricTreeRoots<CircuitTypes<Builder>> to_circuit_type(Builder& builder) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the circuit builder:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(builder, e); };

        PrivateHistoricTreeRoots<CircuitTypes<Builder>> data = {
            to_ct(private_data_tree_root),      to_ct(nullifier_tree_root),         to_ct(contract_tree_root),
            to_ct(l1_to_l2_messages_tree_root), to_ct(private_kernel_vk_tree_root),
        };

        return data;
    };

    template <typename Builder> PrivateHistoricTreeRoots<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Builder>, NCT>::value);
        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Builder>(e); };

        PrivateHistoricTreeRoots<NativeTypes> data = {
            to_nt(private_data_tree_root),      to_nt(nullifier_tree_root),         to_nt(contract_tree_root),
            to_nt(l1_to_l2_messages_tree_root), to_nt(private_kernel_vk_tree_root),
        };

        return data;
    };

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        private_data_tree_root.set_public();
        nullifier_tree_root.set_public();
        contract_tree_root.set_public();
        l1_to_l2_messages_tree_root.set_public();
        private_kernel_vk_tree_root.set_public();
    }
};

template <typename NCT>
std::ostream& operator<<(std::ostream& os, PrivateHistoricTreeRoots<NCT> const& historic_tree_roots)
{
    return os << "private_data_tree_root: " << historic_tree_roots.private_data_tree_root << "\n"
              << "nullifier_tree_root: " << historic_tree_roots.nullifier_tree_root << "\n"
              << "contract_tree_root: " << historic_tree_roots.contract_tree_root << "\n"
              << "l1_to_l2_messages_tree_root: " << historic_tree_roots.l1_to_l2_messages_tree_root << "\n"
              << "private_kernel_vk_tree_root: " << historic_tree_roots.private_kernel_vk_tree_root << "\n";
}

}  // namespace aztec3::circuits::abis
