#pragma once
#include "init.hpp"
#include "nullifier_tree_testing_harness.hpp"

#include "aztec3/circuits/abis/public_data_update_request.hpp"
#include "aztec3/circuits/hash.hpp"
#include "aztec3/constants.hpp"

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::rollup::test_utils::utils {

namespace {

using NT = aztec3::utils::types::NativeTypes;

// Helpers
using aztec3::circuits::get_sibling_path;

// Types
using ConstantRollupData = aztec3::circuits::abis::ConstantRollupData<NT>;
using BaseRollupInputs = aztec3::circuits::abis::BaseRollupInputs<NT>;
using MergeRollupInputs = aztec3::circuits::abis::MergeRollupInputs<NT>;
using BaseOrMergeRollupPublicInputs = aztec3::circuits::abis::BaseOrMergeRollupPublicInputs<NT>;
using RootRollupInputs = aztec3::circuits::abis::RootRollupInputs<NT>;
using DummyBuilder = aztec3::utils::DummyCircuitBuilder;

using Aggregator = aztec3::circuits::recursion::Aggregator;
using AppendOnlyTreeSnapshot = aztec3::circuits::abis::AppendOnlyTreeSnapshot<NT>;

using NullifierLeafPreimage = aztec3::circuits::abis::NullifierLeafPreimage<NT>;

// Tree Aliases
using MemoryStore = stdlib::merkle_tree::MemoryStore;
using MerkleTree = stdlib::merkle_tree::MerkleTree<MemoryStore>;
using NullifierTree = stdlib::merkle_tree::NullifierMemoryTree;
using NullifierLeaf = stdlib::merkle_tree::nullifier_leaf;

using KernelData = aztec3::circuits::abis::PreviousKernelData<NT>;

using aztec3::circuits::abis::MembershipWitness;
using aztec3::circuits::abis::PreviousRollupData;

using nullifier_tree_testing_values = std::tuple<BaseRollupInputs, AppendOnlyTreeSnapshot, AppendOnlyTreeSnapshot>;
}  // namespace

BaseRollupInputs base_rollup_inputs_from_kernels(std::array<KernelData, 2> kernel_data);

BaseRollupInputs base_rollup_inputs_from_kernels(std::array<KernelData, 2> kernel_data,
                                                 abis::GlobalVariables<NT> global_variables);

BaseRollupInputs base_rollup_inputs_from_kernels(std::array<KernelData, 2> kernel_data,
                                                 MerkleTree& private_data_tree,
                                                 MerkleTree& contract_tree,
                                                 MerkleTree& public_data_tree,
                                                 MerkleTree& l1_to_l2_msg_tree);

BaseRollupInputs base_rollup_inputs_from_kernels(std::array<KernelData, 2> kernel_data,
                                                 fr prev_global_variables_hash,
                                                 MerkleTree& private_data_tree,
                                                 MerkleTree& nullifier_tree,
                                                 MerkleTree& contract_tree,
                                                 MerkleTree& public_data_tree,
                                                 MerkleTree& l1_to_l2_msg_tree);


template <size_t N> std::array<fr, N> get_sibling_path(MerkleTree& tree, uint256_t leafIndex)
{
    std::array<fr, N> siblingPath;
    auto path = tree.get_hash_path(leafIndex);
    for (size_t i = 0; i < N; i++) {
        if (leafIndex & (uint256_t(1) << i)) {
            siblingPath[i] = path[i].first;
        } else {
            siblingPath[i] = path[i].second;
        }
    }
    return siblingPath;
}

abis::AppendOnlyTreeSnapshot<NT> get_snapshot_of_tree_state(NullifierMemoryTreeTestingHarness nullifier_tree);

nullifier_tree_testing_values generate_nullifier_tree_testing_values_explicit(
    BaseRollupInputs inputs,
    std::array<fr, MAX_NEW_NULLIFIERS_PER_TX * 2> new_nullifiers,
    const std::vector<fr>& initial_values);

nullifier_tree_testing_values generate_nullifier_tree_testing_values(BaseRollupInputs inputs,
                                                                     size_t starting_insertion_value,
                                                                     size_t spacing);

std::array<fr, NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP> get_empty_l1_to_l2_messages();

nullifier_tree_testing_values generate_nullifier_tree_testing_values(
    BaseRollupInputs inputs, std::array<fr, MAX_NEW_NULLIFIERS_PER_TX * 2> new_nullifiers, size_t spacing);

NullifierMemoryTreeTestingHarness get_initial_nullifier_tree_empty();
NullifierMemoryTreeTestingHarness get_initial_nullifier_tree(const std::vector<fr>& initial_values);

KernelData get_empty_kernel();

RootRollupInputs get_root_rollup_inputs(utils::DummyBuilder& builder,
                                        std::array<KernelData, 4> kernel_data,
                                        std::array<fr, NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP> l1_to_l2_messages);

MergeRollupInputs get_merge_rollup_inputs(utils::DummyBuilder& builder, std::array<KernelData, 4> kernel_data);

inline abis::PublicDataUpdateRequest<NT> make_public_data_update_request(fr leaf_index, fr old_value, fr new_value)
{
    return abis::PublicDataUpdateRequest<NT>{
        .leaf_index = leaf_index,
        .old_value = old_value,
        .new_value = new_value,
    };
};

inline abis::PublicDataRead<NT> make_public_read(fr leaf_index, fr value)
{
    return abis::PublicDataRead<NT>{
        .leaf_index = leaf_index,
        .value = value,
    };
}

bool compare_field_hash_to_expected(std::array<fr, NUM_FIELDS_PER_SHA256> field_hash,
                                    std::array<uint8_t, 32> expected_hash);

std::vector<uint8_t> get_empty_calldata_leaf();

}  // namespace aztec3::circuits::rollup::test_utils::utils
