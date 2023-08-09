#pragma once

#include "../append_only_tree_snapshot.hpp"
#include "../global_variables.hpp"

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::abis {

template <typename NCT> struct ConstantRollupData {
    using fr = typename NCT::fr;

    // The very latest roots as at the very beginning of the entire rollup:
    AppendOnlyTreeSnapshot<NCT> start_historic_blocks_tree_roots_snapshot{};

    // Some members of this struct tbd:
    fr private_kernel_vk_tree_root = 0;
    fr public_kernel_vk_tree_root = 0;
    fr base_rollup_vk_hash = 0;
    fr merge_rollup_vk_hash = 0;

    GlobalVariables<NCT> global_variables{};

    MSGPACK_FIELDS(start_historic_blocks_tree_roots_snapshot,
                   private_kernel_vk_tree_root,
                   public_kernel_vk_tree_root,
                   base_rollup_vk_hash,
                   merge_rollup_vk_hash,
                   global_variables);

    bool operator==(ConstantRollupData<NCT> const&) const = default;
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, ConstantRollupData<NCT> const& obj)
{
    return os << "start_historic_blocks_tree_roots_snapshot:\n"
              << obj.start_historic_blocks_tree_roots_snapshot << "\n"
              << "private_kernel_vk_tree_root: " << obj.private_kernel_vk_tree_root << "\n"
              << "public_kernel_vk_tree_root: " << obj.public_kernel_vk_tree_root << "\n"
              << "base_rollup_vk_hash: " << obj.base_rollup_vk_hash << "\n"
              << "merge_rollup_vk_hash: " << obj.merge_rollup_vk_hash << "\n"
              << "global_variables: " << obj.global_variables << "\n";
}

}  // namespace aztec3::circuits::abis
