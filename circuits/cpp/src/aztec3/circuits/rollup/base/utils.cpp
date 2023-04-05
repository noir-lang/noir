#include "aztec3/circuits/rollup/base/nullifier_tree_testing_harness.hpp"
#include "aztec3/circuits/rollup/base/utils.hpp"
#include "aztec3/constants.hpp"
#include "index.hpp"
#include "init.hpp"

#include <aztec3/circuits/kernel/private/utils.hpp>
#include <aztec3/circuits/mock/mock_kernel_circuit.hpp>
#include "aztec3/circuits/abis/private_kernel/new_contract_data.hpp"

namespace {
using NT = aztec3::utils::types::NativeTypes;
using AggregationObject = aztec3::utils::types::NativeTypes::AggregationObject;

using aztec3::circuits::abis::AppendOnlyTreeSnapshot;
using aztec3::circuits::abis::BaseOrMergeRollupPublicInputs;
using aztec3::circuits::abis::BaseRollupInputs;
using aztec3::circuits::abis::ConstantRollupData;
using aztec3::circuits::abis::MembershipWitness;
using aztec3::circuits::abis::NullifierLeafPreimage;
using aztec3::circuits::abis::private_kernel::NewContractData;

using aztec3::circuits::kernel::private_kernel::utils::dummy_previous_kernel_with_vk_proof;

using plonk::TurboComposer;
} // namespace

namespace aztec3::circuits::rollup::base::utils {

BaseRollupInputs<NT> dummy_base_rollup_inputs_with_vk_proof()
{
    // TODO standardize function naming
    ConstantRollupData<NT> constantRollupData;
    constantRollupData.start_tree_of_historic_private_data_tree_roots_snapshot = {
        .root = native_base_rollup::MerkleTree(PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT).root(),
        .next_available_leaf_index = 0,
    };
    constantRollupData.start_tree_of_historic_contract_tree_roots_snapshot = {
        .root = native_base_rollup::MerkleTree(CONTRACT_TREE_ROOTS_TREE_HEIGHT).root(),
        .next_available_leaf_index = 0,
    };

    std::array<NullifierLeafPreimage<NT>, 2 * KERNEL_NEW_NULLIFIERS_LENGTH> low_nullifier_leaf_preimages;
    std::array<MembershipWitness<NT, NULLIFIER_TREE_HEIGHT>, 2 * KERNEL_NEW_NULLIFIERS_LENGTH>
        low_nullifier_membership_witness;

    for (size_t i = 0; i < 2 * KERNEL_NEW_NULLIFIERS_LENGTH; ++i) {
        low_nullifier_leaf_preimages[i] = NullifierLeafPreimage<NT>();
        low_nullifier_membership_witness[i] = MembershipWitness<NT, NULLIFIER_TREE_HEIGHT>();
    }

    std::array<MembershipWitness<NT, PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT>, 2>
        historic_private_data_tree_root_membership_witnesses = {
            MembershipWitness<NT, PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT>(),
            MembershipWitness<NT, PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT>()
        };

    std::array<MembershipWitness<NT, CONTRACT_TREE_ROOTS_TREE_HEIGHT>, 2>
        historic_contract_tree_root_membership_witnesses = { MembershipWitness<NT, CONTRACT_TREE_ROOTS_TREE_HEIGHT>(),
                                                             MembershipWitness<NT, CONTRACT_TREE_ROOTS_TREE_HEIGHT>() };

    // Kernels
    std::array<abis::private_kernel::PreviousKernelData<NT>, 2> kernel_data;
    // grab mocked previous kernel (need a valid vk, proof, aggobj)
    kernel_data[0] = dummy_previous_kernel_with_vk_proof();
    kernel_data[1] = dummy_previous_kernel_with_vk_proof();

    BaseRollupInputs<NT> baseRollupInputs = { .kernel_data = kernel_data,
                                              .start_private_data_tree_snapshot = {
                                                  .root = native_base_rollup::MerkleTree(PRIVATE_DATA_TREE_HEIGHT).root(),
                                                  .next_available_leaf_index = 0,
                                              },
                                              .start_nullifier_tree_snapshot = AppendOnlyTreeSnapshot<NT>(),
                                              .start_contract_tree_snapshot = {
                                                  .root = native_base_rollup::MerkleTree(CONTRACT_TREE_HEIGHT).root(),
                                                  .next_available_leaf_index = 0,
                                              },
                                              .low_nullifier_leaf_preimages = low_nullifier_leaf_preimages,
                                              .low_nullifier_membership_witness = low_nullifier_membership_witness,
                                              .new_commitments_subtree_sibling_path = { 0 },
                                              .new_nullifiers_subtree_sibling_path = { 0 },
                                              .new_contracts_subtree_sibling_path = { 0 },
                                              .historic_private_data_tree_root_membership_witnesses =
                                                  historic_private_data_tree_root_membership_witnesses,
                                              .historic_contract_tree_root_membership_witnesses =
                                                  historic_contract_tree_root_membership_witnesses,
                                              .constants = constantRollupData };

    return baseRollupInputs;
}

NullifierMemoryTreeTestingHarness get_initial_nullifier_tree(size_t spacing = 5)
{
    // Create a nullifier tree with 8 nullifiers, this padding is required so that the default 0 value in an indexed
    // merkle tree does not affect our tests Nullifier tree at the start
    NullifierMemoryTreeTestingHarness nullifier_tree = NullifierMemoryTreeTestingHarness(NULLIFIER_TREE_HEIGHT);
    // Start from 1 as 0 is always inserted
    for (size_t i = 1; i < 8; ++i) {
        // insert 5, 10, 15, 20 ...
        nullifier_tree.update_element(i * spacing);
    }
    return nullifier_tree;
}

std::tuple<BaseRollupInputs<NT>, AppendOnlyTreeSnapshot<NT>, AppendOnlyTreeSnapshot<NT>>
generate_nullifier_tree_testing_values(BaseRollupInputs<NT> inputs,
                                       size_t starting_insertion_value = 0,
                                       size_t spacing = 5)
{
    const size_t NUMBER_OF_NULLIFIERS = KERNEL_NEW_NULLIFIERS_LENGTH * 2;
    std::array<fr, NUMBER_OF_NULLIFIERS> nullifiers;
    for (size_t i = 0; i < NUMBER_OF_NULLIFIERS; ++i) {
        auto insertion_val = (starting_insertion_value + i * spacing);
        nullifiers[i] = fr(insertion_val);
    }
    return generate_nullifier_tree_testing_values(inputs, nullifiers, spacing);
}

std::tuple<BaseRollupInputs<NT>, AppendOnlyTreeSnapshot<NT>, AppendOnlyTreeSnapshot<NT>>
generate_nullifier_tree_testing_values(BaseRollupInputs<NT> rollupInputs,
                                       std::array<fr, KERNEL_NEW_NULLIFIERS_LENGTH * 2> new_nullifiers,
                                       size_t spacing_prefill = 1)
{
    // Generate nullifier tree testing values

    NullifierMemoryTreeTestingHarness nullifier_tree = get_initial_nullifier_tree(spacing_prefill);
    NullifierMemoryTreeTestingHarness parallel_insertion_tree = get_initial_nullifier_tree(spacing_prefill);

    AppendOnlyTreeSnapshot<NT> nullifier_tree_start_snapshot = {
        .root = nullifier_tree.root(),
        .next_available_leaf_index = uint32_t(8),
    };

    const size_t NUMBER_OF_NULLIFIERS = KERNEL_NEW_NULLIFIERS_LENGTH * 2;
    std::array<NullifierLeafPreimage<NT>, NUMBER_OF_NULLIFIERS> new_nullifier_leaves;
    std::array<NullifierLeafPreimage<NT>, NUMBER_OF_NULLIFIERS> low_nullifier_leaves_preimages;
    std::array<MembershipWitness<NT, NULLIFIER_TREE_HEIGHT>, NUMBER_OF_NULLIFIERS>
        low_nullifier_leaves_preimages_witnesses;

    // Calculate the predecessor nullifier pre-images
    // Get insertion values
    std::vector<fr> insertion_values;
    std::vector<fr> insertion_locations;
    std::array<fr, KERNEL_NEW_NULLIFIERS_LENGTH> new_nullifiers_kernel_1;
    std::array<fr, KERNEL_NEW_NULLIFIERS_LENGTH> new_nullifiers_kernel_2;

    for (size_t i = 0; i < NUMBER_OF_NULLIFIERS; ++i) {
        auto insertion_val = new_nullifiers[i];
        if (i < KERNEL_NEW_NULLIFIERS_LENGTH) {
            new_nullifiers_kernel_1[i] = insertion_val;
        } else {
            new_nullifiers_kernel_2[i - KERNEL_NEW_NULLIFIERS_LENGTH] = insertion_val;
        }
        insertion_locations.push_back(NUMBER_OF_NULLIFIERS + i);
        insertion_values.push_back(insertion_val);
        parallel_insertion_tree.update_element(insertion_val);
    }

    // Get the hash paths etc from the insertion values
    auto witnesses_and_preimages = nullifier_tree.circuit_prep_batch_insert(insertion_values, insertion_locations);

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
        NullifierLeafPreimage<NT> preimage = {
            .leaf_value = new_nullifier_leaves_preimages[i].value,
            .next_index = NT::uint32(new_nullifier_leaves_preimages[i].nextIndex),
            .next_value = new_nullifier_leaves_preimages[i].nextValue,
        };
        new_nullifier_leaves[i] = preimage;
    }

    // Get expected root with subtrees inserted correctly
    fr end_root = parallel_insertion_tree.root();

    // Expected end state
    AppendOnlyTreeSnapshot<NT> nullifier_tree_end_snapshot = {
        .root = end_root,
        .next_available_leaf_index = 16,
    };

    // Get the sibling path, we should be able to use the same path to get to the end root
    std::vector<fr> sibling_path = parallel_insertion_tree.get_sibling_path(8);
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

} // namespace aztec3::circuits::rollup::base::utils