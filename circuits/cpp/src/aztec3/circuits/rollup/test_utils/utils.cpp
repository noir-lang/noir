#include "nullifier_tree_testing_harness.hpp"
#include "utils.hpp"
#include "aztec3/constants.hpp"
#include "init.hpp"

#include <aztec3/circuits/kernel/private/utils.hpp>
#include <aztec3/circuits/mock/mock_kernel_circuit.hpp>
#include "aztec3/circuits/abis/new_contract_data.hpp"
#include "aztec3/circuits/abis/rollup/root/root_rollup_public_inputs.hpp"
namespace {
using NT = aztec3::utils::types::NativeTypes;

using ConstantRollupData = aztec3::circuits::abis::ConstantRollupData<NT>;
using BaseRollupInputs = aztec3::circuits::abis::BaseRollupInputs<NT>;
using RootRollupInputs = aztec3::circuits::abis::RootRollupInputs<NT>;
using RootRollupPublicInputs = aztec3::circuits::abis::RootRollupPublicInputs<NT>;
using DummyComposer = aztec3::utils::DummyComposer;

using Aggregator = aztec3::circuits::recursion::Aggregator;
using AppendOnlyTreeSnapshot = aztec3::circuits::abis::AppendOnlyTreeSnapshot<NT>;
using KernelData = aztec3::circuits::abis::PreviousKernelData<NT>;

using NullifierLeafPreimage = aztec3::circuits::abis::NullifierLeafPreimage<NT>;

using MerkleTree = stdlib::merkle_tree::MemoryTree;
using NullifierTree = stdlib::merkle_tree::NullifierMemoryTree;
using NullifierLeaf = stdlib::merkle_tree::nullifier_leaf;

using aztec3::circuits::abis::BaseOrMergeRollupPublicInputs;
using aztec3::circuits::abis::MembershipWitness;
using MergeRollupInputs = aztec3::circuits::abis::MergeRollupInputs<NT>;
using aztec3::circuits::abis::PreviousRollupData;

using nullifier_tree_testing_values = std::tuple<BaseRollupInputs, AppendOnlyTreeSnapshot, AppendOnlyTreeSnapshot>;

using aztec3::circuits::kernel::private_kernel::utils::dummy_previous_kernel;
} // namespace

namespace aztec3::circuits::rollup::test_utils::utils {

// Want some helper functions for generating kernels with some commitments, nullifiers and contracts

KernelData get_empty_kernel()
{
    return dummy_previous_kernel();
}

void set_kernel_nullifiers(KernelData& kernel_data, std::array<fr, KERNEL_NEW_NULLIFIERS_LENGTH> new_nullifiers)
{
    for (size_t i = 0; i < KERNEL_NEW_NULLIFIERS_LENGTH; i++) {
        kernel_data.public_inputs.end.new_nullifiers[i] = new_nullifiers[i];
    }
}

void set_kernel_commitments(KernelData& kernel_data, std::array<fr, KERNEL_NEW_COMMITMENTS_LENGTH> new_commitments)
{
    for (size_t i = 0; i < KERNEL_NEW_COMMITMENTS_LENGTH; i++) {
        kernel_data.public_inputs.end.new_commitments[i] = new_commitments[i];
    }
}

BaseRollupInputs base_rollup_inputs_from_kernels(std::array<KernelData, 2> kernel_data)
{
    // @todo Look at the starting points for all of these.
    // By supporting as inputs we can make very generic tests, where it is trivial to try new setups.
    MerkleTree historic_private_data_tree = MerkleTree(PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT);
    MerkleTree historic_contract_tree = MerkleTree(CONTRACT_TREE_ROOTS_TREE_HEIGHT);
    MerkleTree historic_l1_to_l2_msg_tree = MerkleTree(L1_TO_L2_MSG_TREE_ROOTS_TREE_HEIGHT);

    MerkleTree private_data_tree = MerkleTree(PRIVATE_DATA_TREE_HEIGHT);
    MerkleTree contract_tree = MerkleTree(CONTRACT_TREE_HEIGHT);

    // Historic trees are initialised with an empty root at position 0.
    historic_private_data_tree.update_element(0, private_data_tree.root());
    historic_contract_tree.update_element(0, contract_tree.root());
    historic_l1_to_l2_msg_tree.update_element(0, MerkleTree(L1_TO_L2_MSG_TREE_HEIGHT).root());

    ConstantRollupData constantRollupData = {
        .start_tree_of_historic_private_data_tree_roots_snapshot = {
            .root = historic_private_data_tree.root(),
            .next_available_leaf_index = 1,
        },
        .start_tree_of_historic_contract_tree_roots_snapshot = {
            .root = historic_contract_tree.root(),
            .next_available_leaf_index = 1,
        },
        .tree_of_historic_l1_to_l2_msg_tree_roots_snapshot = {
            .root = historic_l1_to_l2_msg_tree.root(),
            .next_available_leaf_index = 1,
        },
    };

    for (size_t i = 0; i < 2; i++) {
        kernel_data[i].public_inputs.constants.historic_tree_roots.private_historic_tree_roots.private_data_tree_root =
            private_data_tree.root();
        kernel_data[i].public_inputs.constants.historic_tree_roots.private_historic_tree_roots.contract_tree_root =
            contract_tree.root();
        // @todo Add l1 -> l2 root.
    }

    BaseRollupInputs baseRollupInputs = { .kernel_data = kernel_data,
                                              .start_private_data_tree_snapshot = {
                                                  .root = private_data_tree.root(),
                                                  .next_available_leaf_index = 0,
                                              },
                                              .start_contract_tree_snapshot = {
                                                  .root = contract_tree.root(),
                                                  .next_available_leaf_index = 0,
                                              },
                                              .constants = constantRollupData };

    // Initialise nullifier tree with 0..7
    std::vector<fr> initial_values = { 1, 2, 3, 4, 5, 6, 7 };

    std::array<fr, KERNEL_NEW_NULLIFIERS_LENGTH * 2> nullifiers;
    for (size_t i = 0; i < 2; i++) {
        for (size_t j = 0; j < KERNEL_NEW_NULLIFIERS_LENGTH; j++) {
            nullifiers[i * 4 + j] = kernel_data[i].public_inputs.end.new_nullifiers[j];
        }
    }

    // TODO: It is a bit hacky here that it is always the same location we are inserting it.

    auto temp = generate_nullifier_tree_testing_values_explicit(baseRollupInputs, nullifiers, initial_values);
    baseRollupInputs = std::get<0>(temp);

    baseRollupInputs.new_contracts_subtree_sibling_path =
        get_sibling_path<CONTRACT_SUBTREE_INCLUSION_CHECK_DEPTH>(contract_tree, 0, CONTRACT_SUBTREE_DEPTH);

    baseRollupInputs.new_commitments_subtree_sibling_path =
        get_sibling_path<PRIVATE_DATA_SUBTREE_INCLUSION_CHECK_DEPTH>(private_data_tree, 0, PRIVATE_DATA_SUBTREE_DEPTH);

    baseRollupInputs.historic_private_data_tree_root_membership_witnesses[0] = {
        .leaf_index = 0,
        .sibling_path = get_sibling_path<PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT>(historic_private_data_tree, 0, 0),
    };
    baseRollupInputs.historic_private_data_tree_root_membership_witnesses[1] =
        baseRollupInputs.historic_private_data_tree_root_membership_witnesses[0];

    baseRollupInputs.historic_contract_tree_root_membership_witnesses[0] = {
        .leaf_index = 0,
        .sibling_path = get_sibling_path<CONTRACT_TREE_ROOTS_TREE_HEIGHT>(historic_contract_tree, 0, 0),
    };
    baseRollupInputs.historic_contract_tree_root_membership_witnesses[1] =
        baseRollupInputs.historic_contract_tree_root_membership_witnesses[0];

    return baseRollupInputs;
}

std::array<PreviousRollupData<NT>, 2> get_previous_rollup_data(DummyComposer& composer,
                                                               std::array<KernelData, 4> kernel_data)
{
    // NOTE: Still assuming that this is first and second. Don't handle more rollups atm
    auto base_rollup_input_1 = base_rollup_inputs_from_kernels({ kernel_data[0], kernel_data[1] });
    auto base_public_input_1 =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, base_rollup_input_1);

    // Build the trees based on inputs in base_rollup_input_1.
    MerkleTree private_data_tree = MerkleTree(PRIVATE_DATA_TREE_HEIGHT);
    MerkleTree contract_tree = MerkleTree(CONTRACT_TREE_HEIGHT);
    std::vector<fr> initial_values = { 1, 2, 3, 4, 5, 6, 7 };
    std::array<fr, KERNEL_NEW_NULLIFIERS_LENGTH * 2> nullifiers;

    for (size_t i = 0; i < 2; i++) {
        for (size_t j = 0; j < KERNEL_NEW_COMMITMENTS_LENGTH; j++) {
            private_data_tree.update_element(i * KERNEL_NEW_COMMITMENTS_LENGTH + j,
                                             kernel_data[i].public_inputs.end.new_commitments[j]);
        }
        auto contract_data = kernel_data[i].public_inputs.end.new_contracts[0];
        auto contract_leaf = crypto::pedersen_commitment::compress_native(
            { contract_data.contract_address, contract_data.portal_contract_address, contract_data.function_tree_root },
            GeneratorIndex::CONTRACT_LEAF);
        contract_tree.update_element(i, contract_leaf);
        for (size_t j = 0; j < KERNEL_NEW_NULLIFIERS_LENGTH; j++) {
            initial_values.push_back(kernel_data[i].public_inputs.end.new_nullifiers[j]);
            nullifiers[i * KERNEL_NEW_NULLIFIERS_LENGTH + j] = kernel_data[2 + i].public_inputs.end.new_nullifiers[j];
        }
    }

    auto base_rollup_input_2 = base_rollup_inputs_from_kernels({ kernel_data[2], kernel_data[3] });
    auto temp = generate_nullifier_tree_testing_values_explicit(base_rollup_input_2, nullifiers, initial_values);
    base_rollup_input_2 = std::get<0>(temp);

    base_rollup_input_2.start_private_data_tree_snapshot = base_public_input_1.end_private_data_tree_snapshot;
    base_rollup_input_2.start_nullifier_tree_snapshot = base_public_input_1.end_nullifier_tree_snapshot;
    base_rollup_input_2.start_contract_tree_snapshot = base_public_input_1.end_contract_tree_snapshot;

    // @todo Need an additional tests to check that these below are correct.
    // Changing the index in private tree still pass tests etc (16).
    base_rollup_input_2.new_contracts_subtree_sibling_path =
        get_sibling_path<CONTRACT_SUBTREE_INCLUSION_CHECK_DEPTH>(contract_tree, 1, CONTRACT_SUBTREE_DEPTH);
    base_rollup_input_2.new_commitments_subtree_sibling_path =
        get_sibling_path<PRIVATE_DATA_SUBTREE_INCLUSION_CHECK_DEPTH>(private_data_tree, 8, PRIVATE_DATA_SUBTREE_DEPTH);

    auto base_public_input_2 =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, base_rollup_input_2);

    PreviousRollupData<NT> previous_rollup1 = {
        .base_or_merge_rollup_public_inputs = base_public_input_1,
        .proof = kernel_data[0].proof,
        .vk = kernel_data[0].vk,
        .vk_index = 0,
        .vk_sibling_path = MembershipWitness<NT, ROLLUP_VK_TREE_HEIGHT>(),
    };
    PreviousRollupData<NT> previous_rollup2 = {
        .base_or_merge_rollup_public_inputs = base_public_input_2,
        .proof = kernel_data[2].proof,
        .vk = kernel_data[2].vk,
        .vk_index = 0,
        .vk_sibling_path = MembershipWitness<NT, ROLLUP_VK_TREE_HEIGHT>(),
    };

    return { previous_rollup1, previous_rollup2 };
}

MergeRollupInputs get_merge_rollup_inputs(utils::DummyComposer& composer, std::array<KernelData, 4> kernel_data)
{
    MergeRollupInputs inputs = { .previous_rollup_data = get_previous_rollup_data(composer, kernel_data) };
    return inputs;
}

RootRollupInputs get_root_rollup_inputs(utils::DummyComposer& composer, std::array<KernelData, 4> kernel_data)
{
    MerkleTree historic_private_data_tree = MerkleTree(PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT);
    MerkleTree historic_contract_tree = MerkleTree(CONTRACT_TREE_ROOTS_TREE_HEIGHT);
    MerkleTree historic_l1_to_l2_msg_tree = MerkleTree(L1_TO_L2_MSG_TREE_ROOTS_TREE_HEIGHT);

    MerkleTree private_data_tree = MerkleTree(PRIVATE_DATA_TREE_HEIGHT);
    MerkleTree contract_tree = MerkleTree(CONTRACT_TREE_HEIGHT);

    // Historic trees are initialised with an empty root at position 0.
    historic_private_data_tree.update_element(0, private_data_tree.root());
    historic_contract_tree.update_element(0, contract_tree.root());
    historic_l1_to_l2_msg_tree.update_element(0, MerkleTree(L1_TO_L2_MSG_TREE_HEIGHT).root());

    auto historic_data_sibling_path =
        get_sibling_path<PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT>(historic_private_data_tree, 1, 0);
    auto historic_contract_sibling_path =
        get_sibling_path<CONTRACT_TREE_ROOTS_TREE_HEIGHT>(historic_contract_tree, 1, 0);

    RootRollupInputs rootRollupInputs = {
        .previous_rollup_data = get_previous_rollup_data(composer, kernel_data),
        .new_historic_private_data_tree_root_sibling_path = historic_data_sibling_path,
        .new_historic_contract_tree_root_sibling_path = historic_contract_sibling_path,
    };
    return rootRollupInputs;
}

//////////////////////////
// NULLIFIER TREE BELOW //
//////////////////////////

/**
 * @brief Get initial nullifier tree object
 *
 * @param initial_values values to pre-populate the tree
 * @return NullifierMemoryTreeTestingHarness
 */
NullifierMemoryTreeTestingHarness get_initial_nullifier_tree(std::vector<fr> initial_values)
{
    NullifierMemoryTreeTestingHarness nullifier_tree = NullifierMemoryTreeTestingHarness(NULLIFIER_TREE_HEIGHT);
    for (size_t i = 0; i < initial_values.size(); ++i) {
        nullifier_tree.update_element(initial_values[i]);
    }
    return nullifier_tree;
}

nullifier_tree_testing_values generate_nullifier_tree_testing_values(BaseRollupInputs inputs,
                                                                     size_t starting_insertion_value = 0,
                                                                     size_t spacing = 5)
{
    const size_t NUMBER_OF_NULLIFIERS = KERNEL_NEW_NULLIFIERS_LENGTH * 2;
    std::array<fr, NUMBER_OF_NULLIFIERS> nullifiers;
    for (size_t i = 0; i < NUMBER_OF_NULLIFIERS; ++i) {
        auto insertion_val = (starting_insertion_value + i * spacing);
        nullifiers[i] = fr(insertion_val);
    }

    // Generate initial values lin spaved
    std::vector<fr> initial_values;
    for (size_t i = 1; i < 8; ++i) {
        initial_values.push_back(i * spacing);
    }

    return utils::generate_nullifier_tree_testing_values_explicit(inputs, nullifiers, initial_values);
}

nullifier_tree_testing_values generate_nullifier_tree_testing_values(
    BaseRollupInputs inputs, std::array<fr, KERNEL_NEW_NULLIFIERS_LENGTH * 2> new_nullifiers, size_t spacing = 5)
{
    // Generate initial values lin spaced
    std::vector<fr> initial_values;
    for (size_t i = 1; i < 8; ++i) {
        initial_values.push_back(i * spacing);
    }

    return utils::generate_nullifier_tree_testing_values_explicit(inputs, new_nullifiers, initial_values);
}

nullifier_tree_testing_values generate_nullifier_tree_testing_values_explicit(
    BaseRollupInputs rollupInputs,
    std::array<fr, KERNEL_NEW_NULLIFIERS_LENGTH * 2> new_nullifiers,
    std::vector<fr> initial_values)
{
    size_t start_tree_size = initial_values.size() + 1;
    // Generate nullifier tree testing values
    NullifierMemoryTreeTestingHarness nullifier_tree = get_initial_nullifier_tree(initial_values);
    NullifierMemoryTreeTestingHarness reference_tree = get_initial_nullifier_tree(initial_values);

    AppendOnlyTreeSnapshot nullifier_tree_start_snapshot = {
        .root = nullifier_tree.root(),
        .next_available_leaf_index = uint32_t(start_tree_size),
    };

    const size_t NUMBER_OF_NULLIFIERS = KERNEL_NEW_NULLIFIERS_LENGTH * 2;
    std::array<NullifierLeafPreimage, NUMBER_OF_NULLIFIERS> new_nullifier_leaves;
    std::array<MembershipWitness<NT, NULLIFIER_TREE_HEIGHT>, NUMBER_OF_NULLIFIERS>
        low_nullifier_leaves_preimages_witnesses;

    // Calculate the predecessor nullifier pre-images
    // Get insertion values
    std::vector<fr> insertion_values;
    std::array<fr, KERNEL_NEW_NULLIFIERS_LENGTH> new_nullifiers_kernel_1;
    std::array<fr, KERNEL_NEW_NULLIFIERS_LENGTH> new_nullifiers_kernel_2;

    for (size_t i = 0; i < NUMBER_OF_NULLIFIERS; ++i) {
        auto insertion_val = new_nullifiers[i];
        if (i < KERNEL_NEW_NULLIFIERS_LENGTH) {
            new_nullifiers_kernel_1[i] = insertion_val;
        } else {
            new_nullifiers_kernel_2[i - KERNEL_NEW_NULLIFIERS_LENGTH] = insertion_val;
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
    std::array<MembershipWitness<NT, NULLIFIER_TREE_HEIGHT>, NUMBER_OF_NULLIFIERS> new_membership_witnesses;
    for (size_t i = 0; i < NUMBER_OF_NULLIFIERS; i++) {
        // create an array of the witness from the depth
        std::array<fr, NULLIFIER_TREE_HEIGHT> witness_array;
        std::copy(new_nullifier_leaves_sibling_paths[i].begin(),
                  new_nullifier_leaves_sibling_paths[i].end(),
                  witness_array.begin());

        MembershipWitness<NT, NULLIFIER_TREE_HEIGHT> witness = {
            .leaf_index = NT::uint32(new_nullifier_leave_indexes[i]),
            .sibling_path = witness_array,
        };
        new_membership_witnesses[i] = witness;

        // Create circuit compatible preimages - issue created to remove this step
        NullifierLeafPreimage preimage = {
            .leaf_value = new_nullifier_leaves_preimages[i].value,
            .next_index = NT::uint32(new_nullifier_leaves_preimages[i].nextIndex),
            .next_value = new_nullifier_leaves_preimages[i].nextValue,
        };
        new_nullifier_leaves[i] = preimage;
    }

    // Get expected root with subtrees inserted correctly
    // Expected end state
    AppendOnlyTreeSnapshot nullifier_tree_end_snapshot = {
        .root = reference_tree.root(),
        .next_available_leaf_index = uint32_t(reference_tree.size()),
    };

    std::vector<fr> sibling_path = reference_tree.get_sibling_path(start_tree_size);
    std::array<fr, NULLIFIER_SUBTREE_INCLUSION_CHECK_DEPTH> sibling_path_array;

    // Chop the first 3 levels from the sibling_path
    sibling_path.erase(sibling_path.begin(), sibling_path.begin() + NULLIFIER_SUBTREE_DEPTH);
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

} // namespace aztec3::circuits::rollup::test_utils::utils