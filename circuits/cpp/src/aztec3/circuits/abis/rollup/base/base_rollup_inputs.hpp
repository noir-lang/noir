#pragma once
#include "../../append_only_tree_snapshot.hpp"
#include "../../membership_witness.hpp"
#include "../../previous_kernel_data.hpp"
#include "../constant_rollup_data.hpp"
#include "../nullifier_leaf_preimage.hpp"

#include "aztec3/constants.hpp"

#include <barretenberg/serialize/msgpack.hpp>

#include <math.h>


namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using std::is_same;

template <typename NCT> struct BaseRollupInputs {
    using fr = typename NCT::fr;

    std::array<PreviousKernelData<NCT>, 2> kernel_data;

    AppendOnlyTreeSnapshot<NCT> start_private_data_tree_snapshot;
    AppendOnlyTreeSnapshot<NCT> start_nullifier_tree_snapshot;
    AppendOnlyTreeSnapshot<NCT> start_contract_tree_snapshot;
    fr start_public_data_tree_root;

    std::array<NullifierLeafPreimage<NCT>, 2 * KERNEL_NEW_NULLIFIERS_LENGTH> low_nullifier_leaf_preimages;
    std::array<MembershipWitness<NCT, NULLIFIER_TREE_HEIGHT>, 2 * KERNEL_NEW_NULLIFIERS_LENGTH>
        low_nullifier_membership_witness;

    // For inserting the new subtrees into their respective trees:
    // Note: the insertion leaf index can be derived from the above snapshots' `next_available_leaf_index` values.
    std::array<fr, PRIVATE_DATA_SUBTREE_INCLUSION_CHECK_DEPTH> new_commitments_subtree_sibling_path;
    std::array<fr, NULLIFIER_SUBTREE_INCLUSION_CHECK_DEPTH> new_nullifiers_subtree_sibling_path;
    std::array<fr, CONTRACT_SUBTREE_INCLUSION_CHECK_DEPTH> new_contracts_subtree_sibling_path;
    std::array<MembershipWitness<NCT, PUBLIC_DATA_TREE_HEIGHT>, 2 * KERNEL_PUBLIC_DATA_UPDATE_REQUESTS_LENGTH>
        new_public_data_update_requests_sibling_paths;
    std::array<MembershipWitness<NCT, PUBLIC_DATA_TREE_HEIGHT>, 2 * KERNEL_PUBLIC_DATA_READS_LENGTH>
        new_public_data_reads_sibling_paths;

    std::array<MembershipWitness<NCT, PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT>, 2>
        historic_private_data_tree_root_membership_witnesses;
    std::array<MembershipWitness<NCT, CONTRACT_TREE_ROOTS_TREE_HEIGHT>, 2>
        historic_contract_tree_root_membership_witnesses;
    std::array<MembershipWitness<NCT, L1_TO_L2_MSG_TREE_ROOTS_TREE_HEIGHT>, 2>
        historic_l1_to_l2_msg_tree_root_membership_witnesses;

    ConstantRollupData<NCT> constants;

    // for serialization, update with new fields
    MSGPACK_FIELDS(kernel_data,
                   start_private_data_tree_snapshot,
                   start_nullifier_tree_snapshot,
                   start_contract_tree_snapshot,
                   start_public_data_tree_root,
                   low_nullifier_leaf_preimages,
                   low_nullifier_membership_witness,
                   new_commitments_subtree_sibling_path,
                   new_nullifiers_subtree_sibling_path,
                   new_contracts_subtree_sibling_path,
                   new_public_data_update_requests_sibling_paths,
                   new_public_data_reads_sibling_paths,
                   historic_private_data_tree_root_membership_witnesses,
                   historic_contract_tree_root_membership_witnesses,
                   historic_l1_to_l2_msg_tree_root_membership_witnesses,
                   constants);
    bool operator==(BaseRollupInputs<NCT> const&) const = default;
};

template <typename NCT> void read(uint8_t const*& it, BaseRollupInputs<NCT>& obj)
{
    using serialize::read;

    read(it, obj.kernel_data);
    read(it, obj.start_private_data_tree_snapshot);
    read(it, obj.start_nullifier_tree_snapshot);
    read(it, obj.start_contract_tree_snapshot);
    read(it, obj.start_public_data_tree_root);
    read(it, obj.low_nullifier_leaf_preimages);
    read(it, obj.low_nullifier_membership_witness);
    read(it, obj.new_commitments_subtree_sibling_path);
    read(it, obj.new_nullifiers_subtree_sibling_path);
    read(it, obj.new_contracts_subtree_sibling_path);
    read(it, obj.new_public_data_update_requests_sibling_paths);
    read(it, obj.new_public_data_reads_sibling_paths);
    read(it, obj.historic_private_data_tree_root_membership_witnesses);
    read(it, obj.historic_contract_tree_root_membership_witnesses);
    read(it, obj.historic_l1_to_l2_msg_tree_root_membership_witnesses);
    read(it, obj.constants);
};

template <typename NCT> void write(std::vector<uint8_t>& buf, BaseRollupInputs<NCT> const& obj)
{
    using serialize::write;

    write(buf, obj.kernel_data);
    write(buf, obj.start_private_data_tree_snapshot);
    write(buf, obj.start_nullifier_tree_snapshot);
    write(buf, obj.start_contract_tree_snapshot);
    write(buf, obj.start_public_data_tree_root);
    write(buf, obj.low_nullifier_leaf_preimages);
    write(buf, obj.low_nullifier_membership_witness);
    write(buf, obj.new_commitments_subtree_sibling_path);
    write(buf, obj.new_nullifiers_subtree_sibling_path);
    write(buf, obj.new_contracts_subtree_sibling_path);
    write(buf, obj.new_public_data_update_requests_sibling_paths);
    write(buf, obj.new_public_data_reads_sibling_paths);
    write(buf, obj.historic_private_data_tree_root_membership_witnesses);
    write(buf, obj.historic_contract_tree_root_membership_witnesses);
    write(buf, obj.historic_l1_to_l2_msg_tree_root_membership_witnesses);
    write(buf, obj.constants);
};

template <typename NCT> std::ostream& operator<<(std::ostream& os, BaseRollupInputs<NCT> const& obj)
{
    return os << "kernel_data:\n"
              << obj.kernel_data << "\n"
              << "start_private_data_tree_snapshot:\n"
              << obj.start_private_data_tree_snapshot << "\n"
              << "start_nullifier_tree_snapshot:\n"
              << obj.start_nullifier_tree_snapshot << "\n"
              << "start_contract_tree_snapshot:\n"
              << obj.start_contract_tree_snapshot << "\n"
              << "start_public_data_tree_root:\n"
              << obj.start_public_data_tree_root << "\n"
              << "low_nullifier_leaf_preimages:\n"
              << obj.low_nullifier_leaf_preimages << "\n"
              << "low_nullifier_membership_witness:\n"
              << obj.low_nullifier_membership_witness << "\n"
              << "new_commitments_subtree_sibling_path:\n"
              << obj.new_commitments_subtree_sibling_path << "\n"
              << "new_nullifiers_subtree_sibling_path:\n"
              << obj.new_nullifiers_subtree_sibling_path << "\n"
              << "new_contracts_subtree_sibling_path:\n"
              << obj.new_contracts_subtree_sibling_path << "\n"
              << "new_public_data_update_requests_sibling_paths:\n"
              << obj.new_public_data_update_requests_sibling_paths << "\n"
              << "new_state_reads_sibling_paths:\n"
              << obj.new_public_data_reads_sibling_paths << "\n"
              << "historic_private_data_tree_root_membership_witnesses:\n"
              << obj.historic_private_data_tree_root_membership_witnesses << "\n"
              << "historic_contract_tree_root_membership_witnesses:\n"
              << obj.historic_contract_tree_root_membership_witnesses << "\n"
              << "historic_l1_to_l2_msg_tree_root_membership_witnesses:\n"
              << obj.historic_l1_to_l2_msg_tree_root_membership_witnesses << "\n"
              << "constants:\n"
              << obj.constants << "\n";
}

}  // namespace aztec3::circuits::abis
