#include "aztec3/circuits/rollup/base/nullifier_tree_testing_harness.hpp"
#include "aztec3/circuits/rollup/base/utils.hpp"
#include "aztec3/constants.hpp"
#include "index.hpp"
#include "init.hpp"

#include <aztec3/circuits/kernel/private/utils.hpp>
#include <aztec3/circuits/mock/mock_kernel_circuit.hpp>
#include "aztec3/circuits/abis/new_contract_data.hpp"

namespace {
using NT = aztec3::utils::types::NativeTypes;
using AggregationObject = aztec3::utils::types::NativeTypes::AggregationObject;

using aztec3::circuits::abis::AppendOnlyTreeSnapshot;
using aztec3::circuits::abis::BaseRollupInputs;
using aztec3::circuits::abis::ConstantRollupData;
using aztec3::circuits::abis::MembershipWitness;
using aztec3::circuits::abis::NullifierLeafPreimage;

using aztec3::circuits::kernel::private_kernel::utils::dummy_previous_kernel;
} // namespace

namespace aztec3::circuits::rollup::base::utils {

BaseRollupInputs<NT> dummy_base_rollup_inputs()
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
    // constantRollupData.tree_of_historic_l1_to_l2_msg_tree_roots_snapshot =

    // Kernels
    std::array<abis::PreviousKernelData<NT>, 2> kernel_data;
    // grab mocked previous kernel (need a valid vk, proof, aggobj)
    kernel_data[0] = dummy_previous_kernel();
    kernel_data[1] = dummy_previous_kernel();

    BaseRollupInputs<NT> baseRollupInputs = { .kernel_data = kernel_data,
                                              .start_private_data_tree_snapshot = {
                                                  .root = native_base_rollup::MerkleTree(PRIVATE_DATA_TREE_HEIGHT).root(),
                                                  .next_available_leaf_index = 0,
                                              },
                                              //.start_nullifier_tree_snapshot =
                                              .start_contract_tree_snapshot = {
                                                  .root = native_base_rollup::MerkleTree(CONTRACT_TREE_HEIGHT).root(),
                                                  .next_available_leaf_index = 0,
                                              },
                                              .constants = constantRollupData };

    return baseRollupInputs;
}

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

/**
 * @brief An extension of `get_initial_nullifier_tree` that will populate with linearly spaced values
 *
 * @param spacing
 * @return NullifierMemoryTreeTestingHarness
 */
NullifierMemoryTreeTestingHarness get_initial_nullifier_tree_lin_space(size_t spacing = 5, size_t start = 0)
{
    std::vector<fr> nullifiers;
    for (size_t i = 1; i < 8; ++i) {
        // insert 5, 10, 15, 20 ...
        nullifiers.push_back(start + (i * spacing));
    }
    return get_initial_nullifier_tree(nullifiers);
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

    // Generate initial values lin spaved
    std::vector<fr> initial_values;
    for (size_t i = 1; i < 8; ++i) {
        initial_values.push_back(i * spacing);
    }

    return generate_nullifier_tree_testing_values(inputs, nullifiers, initial_values);
}

std::tuple<BaseRollupInputs<NT>, AppendOnlyTreeSnapshot<NT>, AppendOnlyTreeSnapshot<NT>>
generate_nullifier_tree_testing_values(BaseRollupInputs<NT> inputs,
                                       std::array<fr, KERNEL_NEW_NULLIFIERS_LENGTH * 2> new_nullifiers,
                                       size_t spacing = 5)
{
    // Generate initial values lin spaced
    std::vector<fr> initial_values;
    for (size_t i = 1; i < 8; ++i) {
        initial_values.push_back(i * spacing);
    }

    return generate_nullifier_tree_testing_values(inputs, new_nullifiers, initial_values);
}

std::tuple<BaseRollupInputs<NT>, AppendOnlyTreeSnapshot<NT>, AppendOnlyTreeSnapshot<NT>>
generate_nullifier_tree_testing_values(BaseRollupInputs<NT> rollupInputs,
                                       std::array<fr, KERNEL_NEW_NULLIFIERS_LENGTH * 2> new_nullifiers,
                                       std::vector<fr> initial_values)
{
    size_t start_tree_size = initial_values.size() + 1;
    // Generate nullifier tree testing values
    NullifierMemoryTreeTestingHarness nullifier_tree = get_initial_nullifier_tree(initial_values);
    NullifierMemoryTreeTestingHarness reference_tree = get_initial_nullifier_tree(initial_values);

    AppendOnlyTreeSnapshot<NT> nullifier_tree_start_snapshot = {
        .root = nullifier_tree.root(),
        .next_available_leaf_index = uint32_t(start_tree_size),
    };

    const size_t NUMBER_OF_NULLIFIERS = KERNEL_NEW_NULLIFIERS_LENGTH * 2;
    std::array<NullifierLeafPreimage<NT>, NUMBER_OF_NULLIFIERS> new_nullifier_leaves;
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
        NullifierLeafPreimage<NT> preimage = {
            .leaf_value = new_nullifier_leaves_preimages[i].value,
            .next_index = NT::uint32(new_nullifier_leaves_preimages[i].nextIndex),
            .next_value = new_nullifier_leaves_preimages[i].nextValue,
        };
        new_nullifier_leaves[i] = preimage;
    }

    // Get expected root with subtrees inserted correctly
    // Expected end state
    AppendOnlyTreeSnapshot<NT> nullifier_tree_end_snapshot = {
        .root = reference_tree.root(),
        .next_available_leaf_index = uint32_t(reference_tree.size()),
    };

    // Get the sibling path, we should be able to use the same path to get to the end root
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

} // namespace aztec3::circuits::rollup::base::utils