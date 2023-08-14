#pragma once
#include "../../append_only_tree_snapshot.hpp"
#include "../../membership_witness.hpp"
#include "../../previous_kernel_data.hpp"
#include "../constant_rollup_data.hpp"
#include "../nullifier_leaf_preimage.hpp"

#include "aztec3/constants.hpp"

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::abis {

template <typename NCT> struct BaseRollupInputs {
    using fr = typename NCT::fr;

    std::array<PreviousKernelData<NCT>, 2> kernel_data{};

    AppendOnlyTreeSnapshot<NCT> start_private_data_tree_snapshot{};
    AppendOnlyTreeSnapshot<NCT> start_nullifier_tree_snapshot{};
    AppendOnlyTreeSnapshot<NCT> start_contract_tree_snapshot{};
    fr start_public_data_tree_root{};
    AppendOnlyTreeSnapshot<NCT> start_historic_blocks_tree_snapshot{};

    std::array<NullifierLeafPreimage<NCT>, 2 * MAX_NEW_NULLIFIERS_PER_TX> low_nullifier_leaf_preimages{};
    std::array<MembershipWitness<NCT, NULLIFIER_TREE_HEIGHT>, 2 * MAX_NEW_NULLIFIERS_PER_TX>
        low_nullifier_membership_witness{};

    // For inserting the new subtrees into their respective trees:
    // Note: the insertion leaf index can be derived from the above snapshots' `next_available_leaf_index` values.
    std::array<fr, PRIVATE_DATA_SUBTREE_SIBLING_PATH_LENGTH> new_commitments_subtree_sibling_path{};
    std::array<fr, NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH> new_nullifiers_subtree_sibling_path{};
    std::array<fr, CONTRACT_SUBTREE_SIBLING_PATH_LENGTH> new_contracts_subtree_sibling_path{};
    std::array<std::array<fr, PUBLIC_DATA_TREE_HEIGHT>, 2 * MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>
        new_public_data_update_requests_sibling_paths{};
    std::array<std::array<fr, PUBLIC_DATA_TREE_HEIGHT>, 2 * MAX_PUBLIC_DATA_READS_PER_TX>
        new_public_data_reads_sibling_paths{};

    std::array<MembershipWitness<NCT, HISTORIC_BLOCKS_TREE_HEIGHT>, 2> historic_blocks_tree_root_membership_witnesses{};

    ConstantRollupData<NCT> constants{};

    // for serialization, update with new fields
    MSGPACK_FIELDS(kernel_data,
                   start_private_data_tree_snapshot,
                   start_nullifier_tree_snapshot,
                   start_contract_tree_snapshot,
                   start_public_data_tree_root,
                   start_historic_blocks_tree_snapshot,
                   low_nullifier_leaf_preimages,
                   low_nullifier_membership_witness,
                   new_commitments_subtree_sibling_path,
                   new_nullifiers_subtree_sibling_path,
                   new_contracts_subtree_sibling_path,
                   new_public_data_update_requests_sibling_paths,
                   new_public_data_reads_sibling_paths,
                   historic_blocks_tree_root_membership_witnesses,
                   constants);
    bool operator==(BaseRollupInputs<NCT> const&) const = default;
};

}  // namespace aztec3::circuits::abis
