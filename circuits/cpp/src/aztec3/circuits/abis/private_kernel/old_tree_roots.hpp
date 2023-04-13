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
    typedef typename NCT::boolean boolean;

    fr private_data_tree_root = 0;
    fr nullifier_tree_root = 0;
    fr contract_tree_root = 0;
    fr private_kernel_vk_tree_root = 0; // TODO: future enhancement

    boolean operator==(OldTreeRoots<NCT> const& other) const
    {
        return private_data_tree_root == other.private_data_tree_root &&
               nullifier_tree_root == other.nullifier_tree_root && contract_tree_root == other.contract_tree_root &&
               private_kernel_vk_tree_root == other.private_kernel_vk_tree_root;
    };

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

template <typename NCT> void read(uint8_t const*& it, OldTreeRoots<NCT>& old_tree_roots)
{
    using serialize::read;

    read(it, old_tree_roots.private_data_tree_root);
    read(it, old_tree_roots.nullifier_tree_root);
    read(it, old_tree_roots.contract_tree_root);
    read(it, old_tree_roots.private_kernel_vk_tree_root);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, OldTreeRoots<NCT> const& old_tree_roots)
{
    using serialize::write;

    write(buf, old_tree_roots.private_data_tree_root);
    write(buf, old_tree_roots.nullifier_tree_root);
    write(buf, old_tree_roots.contract_tree_root);
    write(buf, old_tree_roots.private_kernel_vk_tree_root);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, OldTreeRoots<NCT> const& old_tree_roots)
{
    return os << "private_data_tree_root: " << old_tree_roots.private_data_tree_root << "\n"
              << "nullifier_tree_root: " << old_tree_roots.nullifier_tree_root << "\n"
              << "contract_tree_root: " << old_tree_roots.contract_tree_root << "\n"
              << "private_kernel_vk_tree_root: " << old_tree_roots.private_kernel_vk_tree_root << "\n";
}

} // namespace aztec3::circuits::abis::private_kernel