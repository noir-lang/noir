#include "utils.hpp"

#include "nullifier_tree_testing_harness.hpp"

#include "aztec3/circuits/abis/global_variables.hpp"
#include "aztec3/circuits/abis/membership_witness.hpp"
#include "aztec3/circuits/abis/new_contract_data.hpp"
#include "aztec3/circuits/abis/rollup/root/root_rollup_public_inputs.hpp"
#include "aztec3/circuits/hash.hpp"
#include "aztec3/circuits/kernel/private/utils.hpp"
#include "aztec3/circuits/rollup/base/init.hpp"
#include "aztec3/constants.hpp"

#include <barretenberg/barretenberg.hpp>

#include <cstddef>
#include <set>
#include <utility>
#include <vector>
namespace {
using NT = aztec3::utils::types::NativeTypes;

using ConstantRollupData = aztec3::circuits::abis::ConstantRollupData<NT>;
using BaseRollupInputs = aztec3::circuits::abis::BaseRollupInputs<NT>;
using RootRollupInputs = aztec3::circuits::abis::RootRollupInputs<NT>;
using RootRollupPublicInputs = aztec3::circuits::abis::RootRollupPublicInputs<NT>;
using DummyCircuitBuilder = aztec3::utils::DummyCircuitBuilder;

using Aggregator = aztec3::circuits::recursion::Aggregator;
using AppendOnlyTreeSnapshot = aztec3::circuits::abis::AppendOnlyTreeSnapshot<NT>;
using KernelData = aztec3::circuits::abis::PreviousKernelData<NT>;

using NullifierLeafPreimage = aztec3::circuits::abis::NullifierLeafPreimage<NT>;

using MemoryStore = stdlib::merkle_tree::MemoryStore;
using MerkleTree = stdlib::merkle_tree::MerkleTree<MemoryStore>;
using NullifierTree = stdlib::merkle_tree::NullifierMemoryTree;
using NullifierLeaf = stdlib::merkle_tree::nullifier_leaf;

using aztec3::circuits::abis::MembershipWitness;
using MergeRollupInputs = aztec3::circuits::abis::MergeRollupInputs<NT>;
using aztec3::circuits::abis::PreviousRollupData;

using nullifier_tree_testing_values = std::tuple<BaseRollupInputs, AppendOnlyTreeSnapshot, AppendOnlyTreeSnapshot>;

using aztec3::circuits::kernel::private_kernel::utils::dummy_previous_kernel;
}  // namespace

namespace aztec3::circuits::rollup::test_utils::utils {

// Want some helper functions for generating kernels with some commitments, nullifiers and contracts

std::vector<uint8_t> get_empty_calldata_leaf()
{
    auto const number_of_inputs =
        (MAX_NEW_COMMITMENTS_PER_TX + MAX_NEW_NULLIFIERS_PER_TX + MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX * 2 +
         MAX_NEW_L2_TO_L1_MSGS_PER_TX + MAX_NEW_CONTRACTS_PER_TX * 3 + NUM_ENCRYPTED_LOGS_HASHES_PER_TX * 2 +
         NUM_UNENCRYPTED_LOGS_HASHES_PER_TX * 2) *
        2;

    // We subtract 4 from inputs size because 1 logs hash is stored in 2 fields and those 2 fields get converted only
    // to 256 bits and there are 4 logs hashes in total.
    auto const size = (number_of_inputs - 4) * 32;
    std::vector<uint8_t> input_data(size, 0);
    return input_data;
}

KernelData get_empty_kernel()
{
    return dummy_previous_kernel();
}

std::array<fr, NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP> get_empty_l1_to_l2_messages()
{
    std::array<fr, NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP> l1_to_l2_messages = { 0 };
    return l1_to_l2_messages;
}

BaseRollupInputs base_rollup_inputs_from_kernels(std::array<KernelData, 2> kernel_data,
                                                 fr prev_global_variables_hash,
                                                 MerkleTree& private_data_tree,
                                                 MerkleTree& nullifier_tree,
                                                 MerkleTree& contract_tree,
                                                 MerkleTree& public_data_tree,
                                                 MerkleTree& l1_to_l2_msg_tree)
{
    // @todo Look at the starting points for all of these.
    // By supporting as inputs we can make very generic tests, where it is trivial to try new setups.
    MemoryStore historic_blocks_tree_store;
    MerkleTree historic_blocks_tree = MerkleTree(historic_blocks_tree_store, HISTORIC_BLOCKS_TREE_HEIGHT);


    BaseRollupInputs baseRollupInputs = { .kernel_data = kernel_data,
                                              .start_private_data_tree_snapshot = {
                                                  .root = private_data_tree.root(),
                                                  .next_available_leaf_index = 0,
                                              },
                                              .start_contract_tree_snapshot = {
                                                  .root = contract_tree.root(),
                                                  .next_available_leaf_index = 0,
                                              }
                                            };


    std::vector<fr> initial_values(2 * MAX_NEW_NULLIFIERS_PER_TX - 1);

    for (size_t i = 0; i < initial_values.size(); i++) {
        initial_values[i] = i + 1;
    }

    std::array<fr, MAX_NEW_NULLIFIERS_PER_TX * 2> nullifiers;
    for (size_t i = 0; i < 2; i++) {
        for (size_t j = 0; j < MAX_NEW_NULLIFIERS_PER_TX; j++) {
            nullifiers[i * MAX_NEW_NULLIFIERS_PER_TX + j] = kernel_data[i].public_inputs.end.new_nullifiers[j];
        }
    }

    // TODO(lasse): It is a bit hacky here that it is always the same location we are inserting it.

    auto temp = generate_nullifier_tree_testing_values_explicit(baseRollupInputs, nullifiers, initial_values);
    baseRollupInputs = std::get<0>(temp);

    baseRollupInputs.new_contracts_subtree_sibling_path =
        get_sibling_path<CONTRACT_SUBTREE_SIBLING_PATH_LENGTH>(contract_tree, 0, CONTRACT_SUBTREE_HEIGHT);

    baseRollupInputs.new_commitments_subtree_sibling_path =
        get_sibling_path<PRIVATE_DATA_SUBTREE_SIBLING_PATH_LENGTH>(private_data_tree, 0, PRIVATE_DATA_SUBTREE_HEIGHT);


    // Update public data tree to generate sibling paths: we first set the initial public data tree to the result of all
    // public data reads and old_values from public data update requests. Note that, if the right tx reads or writes an
    // index that was already processed by the left one, we don't want to reflect that as part of the initial state, so
    // we skip those.
    std::set<uint256_t> visited_indices;
    for (size_t i = 0; i < 2; i++) {
        for (auto public_data_read : kernel_data[i].public_inputs.end.public_data_reads) {
            auto leaf_index = uint256_t(public_data_read.leaf_index);
            if (public_data_read.is_empty() || visited_indices.contains(leaf_index)) {
                continue;
            }
            visited_indices.insert(leaf_index);
            public_data_tree.update_element(leaf_index, public_data_read.value);
        }

        for (auto public_data_update_request : kernel_data[i].public_inputs.end.public_data_update_requests) {
            auto leaf_index = uint256_t(public_data_update_request.leaf_index);
            if (public_data_update_request.is_empty() || visited_indices.contains(leaf_index)) {
                continue;
            }
            visited_indices.insert(leaf_index);
            public_data_tree.update_element(leaf_index, public_data_update_request.old_value);
        }
    }

    baseRollupInputs.start_public_data_tree_root = public_data_tree.root();

    // create the original historic blocks tree leaf
    auto block_hash = compute_block_hash<NT>(prev_global_variables_hash,
                                             private_data_tree.root(),
                                             nullifier_tree.root(),
                                             contract_tree.root(),
                                             l1_to_l2_msg_tree.root(),
                                             public_data_tree.root());
    historic_blocks_tree.update_element(0, block_hash);

    ConstantRollupData const constantRollupData = { .start_historic_blocks_tree_roots_snapshot = {
                                                        .root = historic_blocks_tree.root(),
                                                        .next_available_leaf_index = 1,
                                                    } };
    baseRollupInputs.constants = constantRollupData;

    // Set historic tree roots data in the public inputs.
    for (size_t i = 0; i < 2; i++) {
        kernel_data[i].public_inputs.constants.block_data.private_data_tree_root = private_data_tree.root();
        kernel_data[i].public_inputs.constants.block_data.nullifier_tree_root = nullifier_tree.root();
        kernel_data[i].public_inputs.constants.block_data.nullifier_tree_root = nullifier_tree.root();
        kernel_data[i].public_inputs.constants.block_data.contract_tree_root = contract_tree.root();
        kernel_data[i].public_inputs.constants.block_data.l1_to_l2_messages_tree_root = l1_to_l2_msg_tree.root();
        kernel_data[i].public_inputs.constants.block_data.blocks_tree_root = historic_blocks_tree.root();
        kernel_data[i].public_inputs.constants.block_data.public_data_tree_root = public_data_tree.root();
        kernel_data[i].public_inputs.constants.block_data.global_variables_hash = prev_global_variables_hash;
    }

    // Then we collect all sibling paths for the reads in the left tx, and then apply the update requests while
    // collecting their paths. And then repeat for the right tx.
    for (size_t i = 0; i < 2; i++) {
        for (size_t j = 0; j < MAX_PUBLIC_DATA_READS_PER_TX; j++) {
            auto public_data_read = kernel_data[i].public_inputs.end.public_data_reads[j];
            if (public_data_read.is_empty()) {
                continue;
            }
            auto leaf_index = uint256_t(public_data_read.leaf_index);
            baseRollupInputs.new_public_data_reads_sibling_paths[i * MAX_PUBLIC_DATA_READS_PER_TX + j] =
                get_sibling_path<PUBLIC_DATA_TREE_HEIGHT>(public_data_tree, leaf_index);
        }

        for (size_t j = 0; j < MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX; j++) {
            auto public_data_update_request = kernel_data[i].public_inputs.end.public_data_update_requests[j];
            if (public_data_update_request.is_empty()) {
                continue;
            }
            auto leaf_index = uint256_t(public_data_update_request.leaf_index);
            public_data_tree.update_element(leaf_index, public_data_update_request.new_value);
            baseRollupInputs
                .new_public_data_update_requests_sibling_paths[i * MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX + j] =
                get_sibling_path<PUBLIC_DATA_TREE_HEIGHT>(public_data_tree, leaf_index);
        }
    }

    // Get historic_root sibling paths
    baseRollupInputs.historic_blocks_tree_root_membership_witnesses[0] = {
        .leaf_index = 0,
        .sibling_path = get_sibling_path<HISTORIC_BLOCKS_TREE_HEIGHT>(historic_blocks_tree, 0, 0),
    };
    baseRollupInputs.historic_blocks_tree_root_membership_witnesses[1] =
        baseRollupInputs.historic_blocks_tree_root_membership_witnesses[0];

    baseRollupInputs.kernel_data = kernel_data;

    return baseRollupInputs;
}

BaseRollupInputs base_rollup_inputs_from_kernels(std::array<KernelData, 2> kernel_data)
{
    MemoryStore private_data_store;
    MerkleTree private_data_tree = MerkleTree(private_data_store, PRIVATE_DATA_TREE_HEIGHT);
    MemoryStore contract_tree_store;
    MerkleTree contract_tree = MerkleTree(contract_tree_store, CONTRACT_TREE_HEIGHT);
    MemoryStore l1_to_l2_messages_store;
    MerkleTree l1_to_l2_messages_tree = MerkleTree(l1_to_l2_messages_store, L1_TO_L2_MSG_TREE_HEIGHT);

    MemoryStore public_data_tree_store;
    MerkleTree public_data_tree(public_data_tree_store, PUBLIC_DATA_TREE_HEIGHT);


    return base_rollup_inputs_from_kernels(
        std::move(kernel_data), private_data_tree, contract_tree, public_data_tree, l1_to_l2_messages_tree);
}

BaseRollupInputs base_rollup_inputs_from_kernels(std::array<KernelData, 2> kernel_data,
                                                 abis::GlobalVariables<NT> global_variables)
{
    MemoryStore private_data_store;
    MerkleTree private_data_tree = MerkleTree(private_data_store, PRIVATE_DATA_TREE_HEIGHT);
    MemoryStore nullifier_data_store;
    MerkleTree nullifier_tree = MerkleTree(nullifier_data_store, PRIVATE_DATA_TREE_HEIGHT);
    MemoryStore contract_tree_store;
    MerkleTree contract_tree = MerkleTree(contract_tree_store, CONTRACT_TREE_HEIGHT);
    MemoryStore l1_to_l2_messages_store;
    MerkleTree l1_to_l2_messages_tree = MerkleTree(l1_to_l2_messages_store, L1_TO_L2_MSG_TREE_HEIGHT);

    MemoryStore public_data_tree_store;
    MerkleTree public_data_tree(public_data_tree_store, PUBLIC_DATA_TREE_HEIGHT);


    return base_rollup_inputs_from_kernels(std::move(kernel_data),
                                           global_variables.hash(),
                                           private_data_tree,
                                           nullifier_tree,
                                           contract_tree,
                                           public_data_tree,
                                           l1_to_l2_messages_tree);
}

BaseRollupInputs base_rollup_inputs_from_kernels(std::array<KernelData, 2> kernel_data,
                                                 MerkleTree& private_data_tree,
                                                 MerkleTree& contract_tree,
                                                 MerkleTree& public_data_tree,
                                                 MerkleTree& l1_to_l2_msg_tree)
{
    MemoryStore nullifier_tree_store;
    MerkleTree nullifier_tree = MerkleTree(nullifier_tree_store, NULLIFIER_TREE_HEIGHT);

    abis::GlobalVariables<NT> prev_globals;

    return base_rollup_inputs_from_kernels(std::move(kernel_data),
                                           prev_globals.hash(),
                                           private_data_tree,
                                           nullifier_tree,
                                           contract_tree,
                                           public_data_tree,
                                           l1_to_l2_msg_tree);
}

std::array<PreviousRollupData<NT>, 2> get_previous_rollup_data(DummyBuilder& builder,
                                                               std::array<KernelData, 4> kernel_data)
{
    // NOTE: Still assuming that this is first and second. Don't handle more rollups atm
    auto base_rollup_input_1 = base_rollup_inputs_from_kernels({ kernel_data[0], kernel_data[1] });
    auto base_public_input_1 =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(builder, base_rollup_input_1);

    // Build the trees based on inputs in base_rollup_input_1.
    MemoryStore private_data_store;
    MerkleTree private_data_tree = MerkleTree(private_data_store, PRIVATE_DATA_TREE_HEIGHT);
    MemoryStore contract_tree_store;
    MerkleTree contract_tree = MerkleTree(contract_tree_store, CONTRACT_TREE_HEIGHT);
    std::vector<fr> initial_values(2 * MAX_NEW_NULLIFIERS_PER_TX - 1);

    for (size_t i = 0; i < initial_values.size(); i++) {
        initial_values[i] = i + 1;
    }
    std::array<fr, MAX_NEW_NULLIFIERS_PER_TX * 2> nullifiers;

    for (size_t i = 0; i < 2; i++) {
        for (size_t j = 0; j < MAX_NEW_COMMITMENTS_PER_TX; j++) {
            private_data_tree.update_element(i * MAX_NEW_COMMITMENTS_PER_TX + j,
                                             kernel_data[i].public_inputs.end.new_commitments[j]);
        }
        auto contract_data = kernel_data[i].public_inputs.end.new_contracts[0];
        if (!contract_data.is_empty()) {
            contract_tree.update_element(i, contract_data.hash());
        }
        for (size_t j = 0; j < MAX_NEW_NULLIFIERS_PER_TX; j++) {
            initial_values.push_back(kernel_data[i].public_inputs.end.new_nullifiers[j]);
            nullifiers[i * MAX_NEW_NULLIFIERS_PER_TX + j] = kernel_data[2 + i].public_inputs.end.new_nullifiers[j];
        }
    }

    auto base_rollup_input_2 = base_rollup_inputs_from_kernels({ kernel_data[2], kernel_data[3] });
    auto temp = generate_nullifier_tree_testing_values_explicit(base_rollup_input_2, nullifiers, initial_values);
    base_rollup_input_2 = std::get<0>(temp);

    base_rollup_input_2.start_private_data_tree_snapshot = base_public_input_1.end_private_data_tree_snapshot;
    base_rollup_input_2.start_nullifier_tree_snapshot = base_public_input_1.end_nullifier_tree_snapshot;
    base_rollup_input_2.start_contract_tree_snapshot = base_public_input_1.end_contract_tree_snapshot;

    base_rollup_input_2.new_contracts_subtree_sibling_path =
        get_sibling_path<CONTRACT_SUBTREE_SIBLING_PATH_LENGTH>(contract_tree, 2, CONTRACT_SUBTREE_HEIGHT);
    base_rollup_input_2.new_commitments_subtree_sibling_path =
        get_sibling_path<PRIVATE_DATA_SUBTREE_SIBLING_PATH_LENGTH>(
            private_data_tree, 2 * MAX_NEW_COMMITMENTS_PER_TX, PRIVATE_DATA_SUBTREE_HEIGHT);

    auto base_public_input_2 =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(builder, base_rollup_input_2);

    PreviousRollupData<NT> const previous_rollup1 = {
        .base_or_merge_rollup_public_inputs = base_public_input_1,
        .proof = kernel_data[0].proof,
        .vk = kernel_data[0].vk,
        .vk_index = 0,
        .vk_sibling_path = MembershipWitness<NT, ROLLUP_VK_TREE_HEIGHT>(),
    };
    PreviousRollupData<NT> const previous_rollup2 = {
        .base_or_merge_rollup_public_inputs = base_public_input_2,
        .proof = kernel_data[2].proof,
        .vk = kernel_data[2].vk,
        .vk_index = 0,
        .vk_sibling_path = MembershipWitness<NT, ROLLUP_VK_TREE_HEIGHT>(),
    };

    return { previous_rollup1, previous_rollup2 };
}

MergeRollupInputs get_merge_rollup_inputs(utils::DummyBuilder& builder, std::array<KernelData, 4> kernel_data)
{
    MergeRollupInputs inputs = { .previous_rollup_data = get_previous_rollup_data(builder, std::move(kernel_data)) };
    return inputs;
}


RootRollupInputs get_root_rollup_inputs(utils::DummyBuilder& builder,
                                        std::array<KernelData, 4> kernel_data,
                                        std::array<fr, NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP> l1_to_l2_messages)
{
    abis::GlobalVariables<NT> globals = { 0, 0, 0, 0 };

    MemoryStore private_data_store;
    const MerkleTree private_data_tree(private_data_store, PRIVATE_DATA_TREE_HEIGHT);

    auto nullifier_tree = get_initial_nullifier_tree_empty();

    MemoryStore contract_tree_store;
    const MerkleTree contract_tree(contract_tree_store, CONTRACT_TREE_HEIGHT);

    MemoryStore l1_to_l2_msg_tree_store;
    MerkleTree l1_to_l2_msg_tree(l1_to_l2_msg_tree_store, L1_TO_L2_MSG_TREE_HEIGHT);

    MemoryStore public_data_tree_store;
    MerkleTree public_data_tree(public_data_tree_store, PUBLIC_DATA_TREE_HEIGHT);

    MemoryStore historic_blocks_tree_store;
    MerkleTree historic_blocks_tree(historic_blocks_tree_store, HISTORIC_BLOCKS_TREE_HEIGHT);

    // Start blocks tree
    auto block_hash = compute_block_hash_with_globals(globals,
                                                      private_data_tree.root(),
                                                      nullifier_tree.root(),
                                                      contract_tree.root(),
                                                      l1_to_l2_msg_tree.root(),
                                                      public_data_tree.root());
    historic_blocks_tree.update_element(0, block_hash);

    // Blocks tree snapshots
    AppendOnlyTreeSnapshot const start_historic_blocks_tree_snapshot = {
        .root = historic_blocks_tree.root(),
        .next_available_leaf_index = 1,
    };

    // Blocks tree
    auto blocks_tree_sibling_path = get_sibling_path<HISTORIC_BLOCKS_TREE_HEIGHT>(historic_blocks_tree, 1, 0);

    // l1 to l2 tree
    auto l1_to_l2_tree_sibling_path =
        get_sibling_path<L1_TO_L2_MSG_SUBTREE_SIBLING_PATH_LENGTH>(l1_to_l2_msg_tree, 0, L1_TO_L2_MSG_SUBTREE_HEIGHT);

    // l1_to_l2_message tree snapshots
    AppendOnlyTreeSnapshot const start_l1_to_l2_msg_tree_snapshot = {
        .root = l1_to_l2_msg_tree.root(),
        .next_available_leaf_index = 0,
    };

    RootRollupInputs rootRollupInputs = {
        .previous_rollup_data = get_previous_rollup_data(builder, std::move(kernel_data)),
        .new_l1_to_l2_messages = l1_to_l2_messages,
        .new_l1_to_l2_messages_tree_root_sibling_path = l1_to_l2_tree_sibling_path,
        .start_l1_to_l2_messages_tree_snapshot = start_l1_to_l2_msg_tree_snapshot,
        .start_historic_blocks_tree_snapshot = start_historic_blocks_tree_snapshot,
        .new_historic_blocks_tree_sibling_path = blocks_tree_sibling_path,
    };
    return rootRollupInputs;
}

//////////////////////////
// NULLIFIER TREE BELOW //
//////////////////////////

/**
 * @brief Get initial nullifier tree object
 *
 * @return NullifierMemoryTreeTestingHarness
 */
NullifierMemoryTreeTestingHarness get_initial_nullifier_tree_empty()
{
    NullifierMemoryTreeTestingHarness nullifier_tree = NullifierMemoryTreeTestingHarness(NULLIFIER_TREE_HEIGHT);
    for (size_t i = 0; i < (MAX_NEW_NULLIFIERS_PER_TX * 2 - 1); i++) {
        nullifier_tree.update_element(i + 1);
    }
    return nullifier_tree;
}

/**
 * @brief Get initial nullifier tree object
 *
 * @param initial_values values to pre-populate the tree
 * @return NullifierMemoryTreeTestingHarness
 */
NullifierMemoryTreeTestingHarness get_initial_nullifier_tree(const std::vector<fr>& initial_values)
{
    NullifierMemoryTreeTestingHarness nullifier_tree = NullifierMemoryTreeTestingHarness(NULLIFIER_TREE_HEIGHT);
    for (const auto& initial_value : initial_values) {
        nullifier_tree.update_element(initial_value);
    }
    return nullifier_tree;
}

nullifier_tree_testing_values generate_nullifier_tree_testing_values(BaseRollupInputs inputs,
                                                                     size_t starting_insertion_value = 0,
                                                                     size_t spacing = 5)
{
    const size_t NUMBER_OF_NULLIFIERS = MAX_NEW_NULLIFIERS_PER_TX * 2;
    std::array<fr, NUMBER_OF_NULLIFIERS> nullifiers;
    for (size_t i = 0; i < NUMBER_OF_NULLIFIERS; ++i) {
        auto insertion_val = (starting_insertion_value + i * spacing);
        nullifiers[i] = fr(insertion_val);
    }

    // Generate initial values lin spaved
    std::vector<fr> initial_values;
    for (size_t i = 1; i < NUMBER_OF_NULLIFIERS; ++i) {
        initial_values.emplace_back(i * spacing);
    }

    return utils::generate_nullifier_tree_testing_values_explicit(std::move(inputs), nullifiers, initial_values);
}

nullifier_tree_testing_values generate_nullifier_tree_testing_values(
    BaseRollupInputs inputs, std::array<fr, MAX_NEW_NULLIFIERS_PER_TX * 2> new_nullifiers, size_t spacing = 5)
{
    // Generate initial values lin spaced
    std::vector<fr> initial_values;
    for (size_t i = 1; i < 2 * MAX_NEW_NULLIFIERS_PER_TX; ++i) {
        initial_values.emplace_back(i * spacing);
    }

    return utils::generate_nullifier_tree_testing_values_explicit(std::move(inputs), new_nullifiers, initial_values);
}

nullifier_tree_testing_values generate_nullifier_tree_testing_values_explicit(
    BaseRollupInputs rollupInputs,
    std::array<fr, MAX_NEW_NULLIFIERS_PER_TX * 2> new_nullifiers,
    const std::vector<fr>& initial_values)
{
    size_t const start_tree_size = initial_values.size() + 1;
    // Generate nullifier tree testing values
    NullifierMemoryTreeTestingHarness nullifier_tree = get_initial_nullifier_tree(initial_values);
    NullifierMemoryTreeTestingHarness reference_tree = get_initial_nullifier_tree(initial_values);

    AppendOnlyTreeSnapshot const nullifier_tree_start_snapshot = nullifier_tree.get_snapshot();

    const size_t NUMBER_OF_NULLIFIERS = MAX_NEW_NULLIFIERS_PER_TX * 2;
    std::array<NullifierLeafPreimage, NUMBER_OF_NULLIFIERS> new_nullifier_leaves{};

    // Calculate the predecessor nullifier pre-images
    // Get insertion values
    std::vector<fr> insertion_values;
    std::array<fr, MAX_NEW_NULLIFIERS_PER_TX> new_nullifiers_kernel_1{};
    std::array<fr, MAX_NEW_NULLIFIERS_PER_TX> new_nullifiers_kernel_2{};

    for (size_t i = 0; i < NUMBER_OF_NULLIFIERS; ++i) {
        auto insertion_val = new_nullifiers[i];
        if (i < MAX_NEW_NULLIFIERS_PER_TX) {
            new_nullifiers_kernel_1[i] = insertion_val;
        } else {
            new_nullifiers_kernel_2[i - MAX_NEW_NULLIFIERS_PER_TX] = insertion_val;
        }
        insertion_values.push_back(insertion_val);
        reference_tree.update_element(insertion_val);
    }

    // Get the hash paths etc from the insertion values
    auto witnesses_and_preimages = nullifier_tree.circuit_prep_batch_insert(insertion_values);

    auto new_nullifier_leaves_preimages = std::get<0>(witnesses_and_preimages);
    auto new_nullifier_leaves_sibling_paths = std::get<1>(witnesses_and_preimages);
    auto new_nullifier_leave_indexes = std::get<2>(witnesses_and_preimages);

    // Create witness values from this
    std::array<MembershipWitness<NT, NULLIFIER_TREE_HEIGHT>, NUMBER_OF_NULLIFIERS> new_membership_witnesses{};
    for (size_t i = 0; i < NUMBER_OF_NULLIFIERS; i++) {
        // create an array of the witness from the depth
        std::array<fr, NULLIFIER_TREE_HEIGHT> witness_array{};
        std::copy(new_nullifier_leaves_sibling_paths[i].begin(),
                  new_nullifier_leaves_sibling_paths[i].end(),
                  witness_array.begin());

        MembershipWitness<NT, NULLIFIER_TREE_HEIGHT> const witness = {
            .leaf_index = static_cast<NT::uint32>(new_nullifier_leave_indexes[i]),
            .sibling_path = witness_array,
        };
        new_membership_witnesses[i] = witness;

        // Create circuit compatible preimages - issue created to remove this step
        NullifierLeafPreimage const preimage = {
            .leaf_value = new_nullifier_leaves_preimages[i].value,
            .next_value = new_nullifier_leaves_preimages[i].nextValue,
            .next_index = NT::uint32(new_nullifier_leaves_preimages[i].nextIndex),
        };
        new_nullifier_leaves[i] = preimage;
    }

    // Get expected root with subtrees inserted correctly
    // Expected end state
    AppendOnlyTreeSnapshot const nullifier_tree_end_snapshot = reference_tree.get_snapshot();

    std::vector<fr> sibling_path = reference_tree.get_sibling_path(start_tree_size);
    std::array<fr, NULLIFIER_SUBTREE_SIBLING_PATH_LENGTH> sibling_path_array;

    // Chop the first NULLIFIER-SUBTREE-DEPTH levels from the sibling_path
    sibling_path.erase(sibling_path.begin(), sibling_path.begin() + NULLIFIER_SUBTREE_HEIGHT);
    std::copy(sibling_path.begin(), sibling_path.end(), sibling_path_array.begin());

    // Update our start state
    // Nullifier trees
    rollupInputs.start_nullifier_tree_snapshot = nullifier_tree_start_snapshot;
    rollupInputs.new_nullifiers_subtree_sibling_path = sibling_path_array;

    rollupInputs.kernel_data[0].public_inputs.end.new_nullifiers = new_nullifiers_kernel_1;
    rollupInputs.kernel_data[1].public_inputs.end.new_nullifiers = new_nullifiers_kernel_2;

    rollupInputs.low_nullifier_leaf_preimages = new_nullifier_leaves;
    rollupInputs.low_nullifier_membership_witness = new_membership_witnesses;

    return std::make_tuple(rollupInputs, nullifier_tree_start_snapshot, nullifier_tree_end_snapshot);
}

/**
 * @brief Compares a hash calculated within a circuit (made up of two field elements) against
 *        one generated natively, (32 bytes) and checks if they match
 *
 * @param field_hash
 * @param expected_hash
 * @return true
 * @return false
 */
bool compare_field_hash_to_expected(std::array<fr, NUM_FIELDS_PER_SHA256> field_hash,
                                    std::array<uint8_t, 32> expected_hash)
{
    auto high_buffer = field_hash[0].to_buffer();
    auto low_buffer = field_hash[1].to_buffer();

    std::array<uint8_t, 32> field_expanded_hash;
    for (uint8_t i = 0; i < 16; ++i) {
        field_expanded_hash[i] = high_buffer[16 + i];
        field_expanded_hash[16 + i] = low_buffer[16 + i];
    }

    return expected_hash == field_expanded_hash;
}

}  // namespace aztec3::circuits::rollup::test_utils::utils
