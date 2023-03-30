#pragma once
#include "../append_only_tree_snapshot.hpp"

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using std::is_same;

template <typename NCT> struct ConstantRollupData {

    typedef typename NCT::fr fr;

    // The very latest roots as at the very beginning of the entire rollup:
    AppendOnlyTreeSnapshot<NCT> start_tree_of_historic_private_data_tree_roots_snapshot;
    AppendOnlyTreeSnapshot<NCT> start_tree_of_historic_contract_tree_roots_snapshot;
    AppendOnlyTreeSnapshot<NCT> tree_of_historic_l1_to_l2_msg_tree_roots_snapshot;

    // Some members of this struct tbd:
    fr private_kernel_vk_tree_root;
    fr public_kernel_vk_tree_root;
    fr base_rollup_vk_hash;
    fr merge_rollup_vk_hash;

    bool operator==(ConstantRollupData<NCT> const&) const = default;
};

template <typename NCT> void read(uint8_t const*& it, ConstantRollupData<NCT>& obj)
{
    using serialize::read;

    read(it, obj.start_tree_of_historic_private_data_tree_roots_snapshot);
    read(it, obj.start_tree_of_historic_contract_tree_roots_snapshot);
    read(it, obj.tree_of_historic_l1_to_l2_msg_tree_roots_snapshot);
    read(it, obj.private_kernel_vk_tree_root);
    read(it, obj.public_kernel_vk_tree_root);
    read(it, obj.base_rollup_vk_hash);
    read(it, obj.merge_rollup_vk_hash);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, ConstantRollupData<NCT> const& obj)
{
    using serialize::write;

    write(buf, obj.start_tree_of_historic_private_data_tree_roots_snapshot);
    write(buf, obj.start_tree_of_historic_contract_tree_roots_snapshot);
    write(buf, obj.tree_of_historic_l1_to_l2_msg_tree_roots_snapshot);
    write(buf, obj.private_kernel_vk_tree_root);
    write(buf, obj.public_kernel_vk_tree_root);
    write(buf, obj.base_rollup_vk_hash);
    write(buf, obj.merge_rollup_vk_hash);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, ConstantRollupData<NCT> const& obj)
{
    return os << "start_tree_of_historic_private_data_tree_roots_snapshot:\n "
              << obj.start_tree_of_historic_private_data_tree_roots_snapshot << "\n"
              << "start_tree_of_historic_contract_tree_roots_snapshot:\n"
              << obj.start_tree_of_historic_contract_tree_roots_snapshot << "\n"
              << "tree_of_historic_l1_to_l2_msg_tree_roots_snapshot:\n"
              << obj.tree_of_historic_l1_to_l2_msg_tree_roots_snapshot << "\n"
              << "private_kernel_vk_tree_root: " << obj.private_kernel_vk_tree_root << "\n"
              << "public_kernel_vk_tree_root: " << obj.public_kernel_vk_tree_root << "\n"
              << "base_rollup_vk_hash: " << obj.base_rollup_vk_hash << "\n"
              << "merge_rollup_vk_hash: " << obj.merge_rollup_vk_hash << "\n";
}

} // namespace aztec3::circuits::abis