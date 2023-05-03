#pragma once
#include "init.hpp"
#include "nullifier_tree_testing_harness.hpp"

#include "aztec3/circuits/abis/public_data_transition.hpp"
#include "aztec3/constants.hpp"

#include "barretenberg/numeric/uint256/uint256.hpp"

namespace aztec3::circuits::rollup::test_utils::utils {

namespace {

using NT = aztec3::utils::types::NativeTypes;

// Types
using ConstantRollupData = aztec3::circuits::abis::ConstantRollupData<NT>;
using BaseRollupInputs = aztec3::circuits::abis::BaseRollupInputs<NT>;
using MergeRollupInputs = aztec3::circuits::abis::MergeRollupInputs<NT>;
using BaseOrMergeRollupPublicInputs = aztec3::circuits::abis::BaseOrMergeRollupPublicInputs<NT>;
using RootRollupInputs = aztec3::circuits::abis::RootRollupInputs<NT>;
using DummyComposer = aztec3::utils::DummyComposer;

using Aggregator = aztec3::circuits::recursion::Aggregator;
using AppendOnlyTreeSnapshot = aztec3::circuits::abis::AppendOnlyTreeSnapshot<NT>;

using NullifierLeafPreimage = aztec3::circuits::abis::NullifierLeafPreimage<NT>;

// Nullifier Tree Alias
using MerkleTree = stdlib::merkle_tree::MemoryTree;
using NullifierTree = stdlib::merkle_tree::NullifierMemoryTree;
using NullifierLeaf = stdlib::merkle_tree::nullifier_leaf;
using KernelData = aztec3::circuits::abis::PreviousKernelData<NT>;
using MemoryStore = stdlib::merkle_tree::MemoryStore;
using SparseTree = stdlib::merkle_tree::MerkleTree<MemoryStore>;

using aztec3::circuits::abis::MembershipWitness;
using aztec3::circuits::abis::PreviousRollupData;

using nullifier_tree_testing_values = std::tuple<BaseRollupInputs, AppendOnlyTreeSnapshot, AppendOnlyTreeSnapshot>;
}  // namespace

BaseRollupInputs base_rollup_inputs_from_kernels(std::array<KernelData, 2> kernel_data);

BaseRollupInputs base_rollup_inputs_from_kernels(std::array<KernelData, 2> kernel_data,
                                                 MerkleTree& private_data_tree,
                                                 MerkleTree& contract_tree,
                                                 SparseTree& public_data_tree);

template <size_t N>
std::array<fr, N> get_sibling_path(MerkleTree& tree, size_t leafIndex, size_t const& subtree_depth_to_skip)
{
    std::array<fr, N> siblingPath;
    auto path = tree.get_hash_path(leafIndex);
    // slice out the skip
    leafIndex = leafIndex >> (subtree_depth_to_skip);
    for (size_t i = 0; i < N; i++) {
        if (leafIndex & (1 << i)) {
            siblingPath[i] = path[subtree_depth_to_skip + i].first;
        } else {
            siblingPath[i] = path[subtree_depth_to_skip + i].second;
        }
    }
    return siblingPath;
}

template <size_t N> std::array<fr, N> get_sibling_path(SparseTree& tree, uint256_t leafIndex)
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
    std::array<fr, KERNEL_NEW_NULLIFIERS_LENGTH * 2> new_nullifiers,
    const std::vector<fr>& initial_values);

nullifier_tree_testing_values generate_nullifier_tree_testing_values(BaseRollupInputs inputs,
                                                                     size_t starting_insertion_value,
                                                                     size_t spacing);

std::array<fr, NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP> get_empty_l1_to_l2_messages();

nullifier_tree_testing_values generate_nullifier_tree_testing_values(
    BaseRollupInputs inputs, std::array<fr, KERNEL_NEW_NULLIFIERS_LENGTH * 2> new_nullifiers, size_t spacing);

NullifierMemoryTreeTestingHarness get_initial_nullifier_tree(const std::vector<fr>& initial_values);

KernelData get_empty_kernel();

RootRollupInputs get_root_rollup_inputs(utils::DummyComposer& composer,
                                        std::array<KernelData, 4> kernel_data,
                                        std::array<fr, NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP> l1_to_l2_messages);

void set_kernel_commitments(KernelData& kernel_data, std::array<fr, KERNEL_NEW_COMMITMENTS_LENGTH> new_commitments);

void set_kernel_nullifiers(KernelData& kernel_data, std::array<fr, KERNEL_NEW_NULLIFIERS_LENGTH> new_nullifiers);

MergeRollupInputs get_merge_rollup_inputs(utils::DummyComposer& composer, std::array<KernelData, 4> kernel_data);

inline abis::PublicDataTransition<NT> make_public_write(fr leaf_index, fr old_value, fr new_value)
{
    return abis::PublicDataTransition<NT>{
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

bool compare_field_hash_to_expected(std::array<fr, 2> field_hash, std::array<uint8_t, 32> expected_hash);

}  // namespace aztec3::circuits::rollup::test_utils::utils
