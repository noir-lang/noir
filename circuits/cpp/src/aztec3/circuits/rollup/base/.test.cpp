#include "aztec3/circuits/abis/append_only_tree_snapshot.hpp"
#include "aztec3/circuits/abis/membership_witness.hpp"
#include "aztec3/circuits/abis/new_contract_data.hpp"
#include "aztec3/circuits/abis/previous_kernel_data.hpp"
#include "aztec3/circuits/kernel/private/utils.hpp"
#include "aztec3/circuits/abis/rollup/nullifier_leaf_preimage.hpp"
#include "aztec3/constants.hpp"
#include "barretenberg/crypto/sha256/sha256.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/stdlib/merkle_tree/memory_tree.hpp"

#include "aztec3/circuits/rollup/base/utils.hpp"
#include "index.hpp"
#include "init.hpp"
#include "c_bind.h"

#include <aztec3/circuits/apps/test_apps/escrow/deposit.hpp>
#include <aztec3/circuits/apps/test_apps/basic_contract_deployment/basic_contract_deployment.hpp>

#include <aztec3/circuits/abis/call_context.hpp>
#include <aztec3/circuits/abis/call_stack_item.hpp>
#include <aztec3/circuits/abis/contract_deployment_data.hpp>
#include <aztec3/circuits/abis/function_data.hpp>
#include <aztec3/circuits/abis/signed_tx_request.hpp>
#include <aztec3/circuits/abis/tx_context.hpp>
#include <aztec3/circuits/abis/tx_request.hpp>
#include <aztec3/circuits/abis/private_circuit_public_inputs.hpp>
#include <aztec3/circuits/abis/private_kernel/private_inputs.hpp>
#include <aztec3/circuits/abis/kernel_circuit_public_inputs.hpp>
#include <aztec3/circuits/abis/combined_accumulated_data.hpp>
#include <aztec3/circuits/abis/combined_constant_data.hpp>
#include <aztec3/circuits/abis/combined_historic_tree_roots.hpp>
#include <aztec3/circuits/abis/private_historic_tree_roots.hpp>
#include <aztec3/circuits/abis/private_kernel/globals.hpp>

#include <aztec3/circuits/apps/function_execution_context.hpp>

#include <aztec3/circuits/mock/mock_kernel_circuit.hpp>

#include <barretenberg/common/map.hpp>
#include <barretenberg/common/test.hpp>
#include <cstddef>
#include <cstdint>
#include <gtest/gtest.h>
#include <iostream>
#include <tuple>
#include <vector>

#include "utils.hpp"

// Nullifier tree building lib
#include "./nullifier_tree_testing_harness.hpp"
// #include <aztec3/constants.hpp>
// #include <barretenberg/crypto/pedersen/pedersen.hpp>
// #include <barretenberg/stdlib/hash/pedersen/pedersen.hpp>

namespace {

using aztec3::circuits::abis::CallContext;
using aztec3::circuits::abis::CallStackItem;
using aztec3::circuits::abis::ContractDeploymentData;
using aztec3::circuits::abis::FunctionData;
using aztec3::circuits::abis::OptionalPrivateCircuitPublicInputs;
using aztec3::circuits::abis::PrivateCircuitPublicInputs;
using aztec3::circuits::abis::SignedTxRequest;
using aztec3::circuits::abis::TxContext;
using aztec3::circuits::abis::TxRequest;

using aztec3::circuits::abis::CombinedAccumulatedData;
using aztec3::circuits::abis::CombinedConstantData;
using aztec3::circuits::abis::CombinedHistoricTreeRoots;
using aztec3::circuits::abis::KernelCircuitPublicInputs;
using aztec3::circuits::abis::PreviousKernelData;
using aztec3::circuits::abis::private_kernel::Globals;
using aztec3::circuits::abis::private_kernel::PrivateCallData;
using aztec3::circuits::abis::private_kernel::PrivateInputs;

using aztec3::circuits::apps::test_apps::basic_contract_deployment::constructor;
using aztec3::circuits::apps::test_apps::escrow::deposit;

// using aztec3::circuits::mock::mock_circuit;
using aztec3::circuits::kernel::private_kernel::utils::dummy_previous_kernel;
using aztec3::circuits::mock::mock_kernel_circuit;
using aztec3::circuits::rollup::base::utils::dummy_base_rollup_inputs;
// using aztec3::circuits::mock::mock_kernel_inputs;

using aztec3::circuits::abis::AppendOnlyTreeSnapshot;

using aztec3::circuits::abis::MembershipWitness;
using aztec3::circuits::abis::NullifierLeafPreimage;
using aztec3::circuits::rollup::native_base_rollup::BaseOrMergeRollupPublicInputs;
using aztec3::circuits::rollup::native_base_rollup::BaseRollupInputs;
using aztec3::circuits::rollup::native_base_rollup::ConstantRollupData;
using aztec3::circuits::rollup::native_base_rollup::NT;

using aztec3::circuits::abis::FunctionData;
using aztec3::circuits::abis::NewContractData;
using aztec3::circuits::abis::OptionallyRevealedData;

using DummyComposer = aztec3::utils::DummyComposer;
} // namespace

namespace aztec3::circuits::rollup::base::native_base_rollup_circuit {

class base_rollup_tests : public ::testing::Test {
  protected:
    void run_cbind(BaseRollupInputs& base_rollup_inputs,
                   BaseOrMergeRollupPublicInputs& expected_public_inputs,
                   bool compare_pubins = true)
    {
        info("Retesting via cbinds....");
        // TODO might be able to get rid of proving key buffer
        uint8_t const* pk_buf;
        size_t pk_size = base_rollup__init_proving_key(&pk_buf);
        (void)pk_size;
        // info("Proving key size: ", pk_size);

        // TODO might be able to get rid of verification key buffer
        uint8_t const* vk_buf;
        size_t vk_size = base_rollup__init_verification_key(pk_buf, &vk_buf);
        (void)vk_size;
        // info("Verification key size: ", vk_size);

        std::vector<uint8_t> base_rollup_inputs_vec;
        write(base_rollup_inputs_vec, base_rollup_inputs);

        // uint8_t const* proof_data;
        // size_t proof_data_size;
        uint8_t const* public_inputs_buf;
        // info("simulating circuit via cbind");
        size_t public_inputs_size = base_rollup__sim(base_rollup_inputs_vec.data(), &public_inputs_buf);
        // info("Proof size: ", proof_data_size);
        // info("PublicInputs size: ", public_inputs_size);

        if (compare_pubins) {
            BaseOrMergeRollupPublicInputs public_inputs;
            uint8_t const* public_inputs_buf_tmp = public_inputs_buf;
            read(public_inputs_buf_tmp, public_inputs);
            ASSERT_EQ(public_inputs.calldata_hash.size(), expected_public_inputs.calldata_hash.size());
            for (size_t i = 0; i < public_inputs.calldata_hash.size(); i++) {
                ASSERT_EQ(public_inputs.calldata_hash[i], expected_public_inputs.calldata_hash[i]);
            }

            std::vector<uint8_t> expected_public_inputs_vec;
            write(expected_public_inputs_vec, expected_public_inputs);

            ASSERT_EQ(public_inputs_size, expected_public_inputs_vec.size());
            // Just compare the first 10 bytes of the serialized public outputs
            if (public_inputs_size > 10) {
                // for (size_t 0; i < public_inputs_size; i++) {
                for (size_t i = 0; i < 10; i++) {
                    ASSERT_EQ(public_inputs_buf[i], expected_public_inputs_vec[i]);
                }
            }
        }

        free((void*)pk_buf);
        free((void*)vk_buf);
        // free((void*)proof_data);
        free((void*)public_inputs_buf);
        // info("finished retesting via cbinds...");
    }
};

template <size_t N>
std::array<fr, N> get_sibling_path(stdlib::merkle_tree::MemoryTree tree,
                                   size_t leafIndex,
                                   size_t const& subtree_depth_to_skip)
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

TEST_F(base_rollup_tests, native_no_new_contract_leafs)
{
    DummyComposer composer = DummyComposer();
    // When there are no contract deployments. The contract tree should be inserting 0 leafs, (not empty leafs);
    // Initially, the start_contract_tree_snapshot is empty (leaf is 0. hash it up).
    // Get sibling path of index 0 leaf (for circuit to check membership via sibling path)
    // No contract leaves -> will insert empty tree -> i.e. end_contract_tree_root = start_contract_tree_root

    BaseRollupInputs emptyInputs = dummy_base_rollup_inputs();
    auto empty_contract_tree = native_base_rollup::MerkleTree(CONTRACT_TREE_HEIGHT);
    auto sibling_path_of_0 =
        get_sibling_path<CONTRACT_SUBTREE_INCLUSION_CHECK_DEPTH>(empty_contract_tree, 0, CONTRACT_SUBTREE_DEPTH);
    // Set the new_contracts_subtree_sibling_path
    emptyInputs.new_contracts_subtree_sibling_path = sibling_path_of_0;

    BaseOrMergeRollupPublicInputs outputs =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, emptyInputs);

    AppendOnlyTreeSnapshot<NT> expectedEndContractTreeSnapshot = {
        .root = empty_contract_tree.root(),
        .next_available_leaf_index = 2,
    };
    ASSERT_EQ(outputs.start_contract_tree_snapshot, emptyInputs.start_contract_tree_snapshot);
    ASSERT_EQ(outputs.end_contract_tree_snapshot, expectedEndContractTreeSnapshot);
    run_cbind(emptyInputs, outputs);
}

TEST_F(base_rollup_tests, native_contract_leaf_inserted)
{
    DummyComposer composer = DummyComposer();
    // When there is a contract deployment, the contract tree should be inserting 1 leaf.
    // The remaining leafs should be 0 leafs, (not empty leafs);
    BaseRollupInputs inputs = dummy_base_rollup_inputs();

    // Create a "mock" contract deployment
    NewContractData<NT> new_contract = {
        .contract_address = fr(1),
        .portal_contract_address = fr(3),
        .function_tree_root = fr(2),
    };
    inputs.kernel_data[0].public_inputs.end.new_contracts[0] = new_contract;

    auto empty_contract_tree = native_base_rollup::MerkleTree(CONTRACT_TREE_HEIGHT);
    auto sibling_path_of_0 =
        get_sibling_path<CONTRACT_SUBTREE_INCLUSION_CHECK_DEPTH>(empty_contract_tree, 0, CONTRACT_SUBTREE_DEPTH);
    // Set the new_contracts_subtree_sibling_path
    inputs.new_contracts_subtree_sibling_path = sibling_path_of_0;

    // create expected end contract tree snapshot
    auto expected_contract_leaf = crypto::pedersen_commitment::compress_native(
        { new_contract.contract_address, new_contract.portal_contract_address, new_contract.function_tree_root },
        GeneratorIndex::CONTRACT_LEAF);
    auto expeted_end_contracts_snapshot_tree = stdlib::merkle_tree::MemoryTree(CONTRACT_TREE_HEIGHT);
    expeted_end_contracts_snapshot_tree.update_element(0, expected_contract_leaf);

    AppendOnlyTreeSnapshot<NT> expected_end_contracts_snapshot = {
        .root = expeted_end_contracts_snapshot_tree.root(),
        .next_available_leaf_index = 2,
    };
    BaseOrMergeRollupPublicInputs outputs =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, inputs);

    ASSERT_EQ(outputs.start_contract_tree_snapshot, inputs.start_contract_tree_snapshot);
    ASSERT_EQ(outputs.end_contract_tree_snapshot, expected_end_contracts_snapshot);
    run_cbind(inputs, outputs);
}

TEST_F(base_rollup_tests, native_contract_leaf_inserted_in_non_empty_snapshot_tree)
{
    DummyComposer composer = DummyComposer();
    // Same as before except our start_contract_snapshot_tree is not empty
    BaseRollupInputs inputs = dummy_base_rollup_inputs();

    // Create a "mock" contract deployment
    NewContractData<NT> new_contract = {
        .contract_address = fr(1),
        .portal_contract_address = fr(3),
        .function_tree_root = fr(2),
    };
    inputs.kernel_data[0].public_inputs.end.new_contracts[0] = new_contract;

    auto start_contract_tree_snapshot = native_base_rollup::MerkleTree(CONTRACT_TREE_HEIGHT);
    // insert 12 leaves to the tree (next available leaf index is 12)
    for (size_t i = 0; i < 12; ++i) {
        start_contract_tree_snapshot.update_element(i, fr(i));
    }
    // set the start_contract_tree_snapshot
    inputs.start_contract_tree_snapshot = {
        .root = start_contract_tree_snapshot.root(),
        .next_available_leaf_index = 12,
    };

    // Set the new_contracts_subtree_sibling_path
    auto sibling_path = get_sibling_path<CONTRACT_SUBTREE_INCLUSION_CHECK_DEPTH>(
        start_contract_tree_snapshot, 12, CONTRACT_SUBTREE_DEPTH);
    inputs.new_contracts_subtree_sibling_path = sibling_path;

    // create expected end contract tree snapshot
    auto expected_contract_leaf = crypto::pedersen_commitment::compress_native(
        { new_contract.contract_address, new_contract.portal_contract_address, new_contract.function_tree_root },
        GeneratorIndex::CONTRACT_LEAF);
    auto expeted_end_contracts_snapshot_tree = start_contract_tree_snapshot;
    expeted_end_contracts_snapshot_tree.update_element(12, expected_contract_leaf);

    AppendOnlyTreeSnapshot<NT> expected_end_contracts_snapshot = {
        .root = expeted_end_contracts_snapshot_tree.root(),
        .next_available_leaf_index = 14,
    };
    BaseOrMergeRollupPublicInputs outputs =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, inputs);

    ASSERT_EQ(outputs.start_contract_tree_snapshot, inputs.start_contract_tree_snapshot);
    ASSERT_EQ(outputs.end_contract_tree_snapshot, expected_end_contracts_snapshot);
    run_cbind(inputs, outputs);
}

TEST_F(base_rollup_tests, native_new_commitments_tree)
{
    DummyComposer composer = DummyComposer();
    // Create 4 new mock commitments. Add them to kernel data.
    // Then get sibling path so we can verify insert them into the tree.
    BaseRollupInputs inputs = dummy_base_rollup_inputs();

    std::array<NT::fr, KERNEL_NEW_COMMITMENTS_LENGTH> new_commitments_kernel_0 = { fr(0), fr(1), fr(2), fr(3) };
    std::array<NT::fr, KERNEL_NEW_COMMITMENTS_LENGTH> new_commitments_kernel_1 = { fr(4), fr(5), fr(6), fr(7) };

    inputs.kernel_data[0].public_inputs.end.new_commitments = new_commitments_kernel_0;
    inputs.kernel_data[1].public_inputs.end.new_commitments = new_commitments_kernel_1;

    // get sibling path
    auto start_tree = native_base_rollup::MerkleTree(PRIVATE_DATA_TREE_HEIGHT);
    auto sibling_path =
        get_sibling_path<PRIVATE_DATA_SUBTREE_INCLUSION_CHECK_DEPTH>(start_tree, 0, PRIVATE_DATA_SUBTREE_DEPTH);
    inputs.new_commitments_subtree_sibling_path = sibling_path;

    // create expected commitments snapshot tree
    auto expected_end_commitments_snapshot_tree = start_tree;
    for (size_t i = 0; i < new_commitments_kernel_0.size(); ++i) {
        expected_end_commitments_snapshot_tree.update_element(i, new_commitments_kernel_0[i]);
    }
    for (size_t i = 0; i < new_commitments_kernel_1.size(); ++i) {
        expected_end_commitments_snapshot_tree.update_element(KERNEL_NEW_COMMITMENTS_LENGTH + i,
                                                              new_commitments_kernel_1[i]);
    }
    AppendOnlyTreeSnapshot<NT> expected_end_commitments_snapshot = {
        .root = expected_end_commitments_snapshot_tree.root(),
        .next_available_leaf_index = 8,
    };

    BaseOrMergeRollupPublicInputs outputs =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, inputs);
    ASSERT_EQ(outputs.start_private_data_tree_snapshot, inputs.start_private_data_tree_snapshot);
    ASSERT_EQ(outputs.end_private_data_tree_snapshot, expected_end_commitments_snapshot);
    run_cbind(inputs, outputs);
}

template <size_t N> NT::fr calc_root(NT::fr leaf, NT::uint32 leafIndex, std::array<NT::fr, N> siblingPath)
{
    for (size_t i = 0; i < siblingPath.size(); i++) {
        if (leafIndex & (1 << i)) {
            leaf = proof_system::plonk::stdlib::merkle_tree::hash_pair_native(siblingPath[i], leaf);
        } else {
            leaf = proof_system::plonk::stdlib::merkle_tree::hash_pair_native(leaf, siblingPath[i]);
        }
    }
    return leaf;
}

TEST_F(base_rollup_tests, native_new_nullifier_tree_empty)
{
    /**
     * DESCRIPTION
     */

    // This test checks for insertions of all 0 values
    // In this special case we will not need to provide sibling paths to check insertion of the nullifier values
    // This is because 0 values are not actually inserted into the tree, rather the inserted subtree is left
    // empty to begin with.

    DummyComposer composer = DummyComposer();
    BaseRollupInputs empty_inputs = dummy_base_rollup_inputs();
    std::array<fr, KERNEL_NEW_NULLIFIERS_LENGTH* 2> new_nullifiers = { 0, 0, 0, 0, 0, 0, 0, 0 };
    std::tuple<BaseRollupInputs, AppendOnlyTreeSnapshot<NT>, AppendOnlyTreeSnapshot<NT>> inputs_and_snapshots =
        utils::generate_nullifier_tree_testing_values(empty_inputs, new_nullifiers, 1);

    BaseRollupInputs testing_inputs = std::get<0>(inputs_and_snapshots);
    AppendOnlyTreeSnapshot<NT> nullifier_tree_start_snapshot = std::get<1>(inputs_and_snapshots);
    AppendOnlyTreeSnapshot<NT> nullifier_tree_end_snapshot = std::get<2>(inputs_and_snapshots);

    /**
     * RUN
     */

    // Run the circuit
    BaseOrMergeRollupPublicInputs outputs =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, testing_inputs);

    /**
     * ASSERT
     */
    // Start state
    ASSERT_EQ(outputs.start_nullifier_tree_snapshot, nullifier_tree_start_snapshot);

    // End state
    ASSERT_EQ(outputs.end_nullifier_tree_snapshot, nullifier_tree_end_snapshot);
    ASSERT_EQ(outputs.end_nullifier_tree_snapshot.root, outputs.start_nullifier_tree_snapshot.root);
    ASSERT_EQ(outputs.end_nullifier_tree_snapshot.next_available_leaf_index,
              outputs.start_nullifier_tree_snapshot.next_available_leaf_index + 8);
}

TEST_F(base_rollup_tests, native_new_nullifier_tree_all_larger)
{
    /**
     * SETUP
     */
    DummyComposer composer = DummyComposer();
    BaseRollupInputs empty_inputs = dummy_base_rollup_inputs();
    std::tuple<BaseRollupInputs, AppendOnlyTreeSnapshot<NT>, AppendOnlyTreeSnapshot<NT>> inputs_and_snapshots =
        utils::generate_nullifier_tree_testing_values(empty_inputs, 8, 1);

    BaseRollupInputs testing_inputs = std::get<0>(inputs_and_snapshots);
    AppendOnlyTreeSnapshot<NT> nullifier_tree_start_snapshot = std::get<1>(inputs_and_snapshots);
    AppendOnlyTreeSnapshot<NT> nullifier_tree_end_snapshot = std::get<2>(inputs_and_snapshots);

    /**
     * RUN
     */

    // Run the circuit
    BaseOrMergeRollupPublicInputs outputs =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, testing_inputs);

    /**
     * ASSERT
     */
    // Start state
    ASSERT_EQ(outputs.start_nullifier_tree_snapshot, nullifier_tree_start_snapshot);

    // End state
    ASSERT_EQ(outputs.end_nullifier_tree_snapshot, nullifier_tree_end_snapshot);
}

TEST_F(base_rollup_tests, native_new_nullifier_tree_sparse)
{
    /**
     * DESCRIPTION
     */

    DummyComposer composer = DummyComposer();
    BaseRollupInputs empty_inputs = dummy_base_rollup_inputs();
    std::tuple<BaseRollupInputs, AppendOnlyTreeSnapshot<NT>, AppendOnlyTreeSnapshot<NT>> inputs_and_snapshots =
        utils::generate_nullifier_tree_testing_values(empty_inputs, 1, 5);

    BaseRollupInputs testing_inputs = std::get<0>(inputs_and_snapshots);
    AppendOnlyTreeSnapshot<NT> nullifier_tree_start_snapshot = std::get<1>(inputs_and_snapshots);
    AppendOnlyTreeSnapshot<NT> nullifier_tree_end_snapshot = std::get<2>(inputs_and_snapshots);

    /**
     * RUN
     */

    // Run the circuit
    BaseOrMergeRollupPublicInputs outputs =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, testing_inputs);

    /**
     * ASSERT
     */
    // Start state
    ASSERT_EQ(outputs.start_nullifier_tree_snapshot, nullifier_tree_start_snapshot);

    // End state
    ASSERT_EQ(outputs.end_nullifier_tree_snapshot, nullifier_tree_end_snapshot);
}

TEST_F(base_rollup_tests, native_nullifier_tree_regression)
{
    // Regression test caught when testing the typescript nullifier tree implementation
    DummyComposer composer = DummyComposer();
    BaseRollupInputs empty_inputs = dummy_base_rollup_inputs();

    // This test runs after some data has already been inserted into the tree
    // This test will pre-populate the tree with 24 values (0 item + 23 more) simulating that a rollup inserting two
    // random values has already succeeded. This rollup then adds two further random values that will end up having
    // their low nullifiers point at each other
    std::vector<fr> initial_values(23, 0);
    for (size_t i = 0; i < 7; i++) {
        initial_values[i] = i + 1;
    }
    // Note these are hex representations
    initial_values[7] = uint256_t("2bb9aa4a22a6ae7204f2c67abaab59cead6558cde4ee25ce3464704cb2e38136");
    initial_values[8] = uint256_t("16a732095298ccca828c4d747813f8bd46e188079ed17904e2c9de50760833c8");

    std::array<fr, KERNEL_NEW_NULLIFIERS_LENGTH* 2> new_nullifiers = { 0 };
    new_nullifiers[0] = uint256_t("16da4f27fb78de7e0db4c5a04b569bc46382c5f471da2f7d670beff1614e0118"),
    new_nullifiers[1] = uint256_t("26ab07ce103a55e29f11478eaa36cebd10c4834b143a7debcc7ef53bfdb547dd");

    std::tuple<BaseRollupInputs, AppendOnlyTreeSnapshot<NT>, AppendOnlyTreeSnapshot<NT>> inputs_and_snapshots =
        utils::generate_nullifier_tree_testing_values(empty_inputs, new_nullifiers, initial_values);
    BaseRollupInputs testing_inputs = std::get<0>(inputs_and_snapshots);
    AppendOnlyTreeSnapshot<NT> nullifier_tree_start_snapshot = std::get<1>(inputs_and_snapshots);
    AppendOnlyTreeSnapshot<NT> nullifier_tree_end_snapshot = std::get<2>(inputs_and_snapshots);

    /**
     * RUN
     */

    // Run the circuit
    BaseOrMergeRollupPublicInputs outputs =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, testing_inputs);

    /**
     * ASSERT
     */
    // Start state
    ASSERT_EQ(outputs.start_nullifier_tree_snapshot, nullifier_tree_start_snapshot);

    // End state
    ASSERT_EQ(outputs.end_nullifier_tree_snapshot, nullifier_tree_end_snapshot);
}

void perform_standard_nullifier_test(std::array<fr, KERNEL_NEW_NULLIFIERS_LENGTH * 2> new_nullifiers)
{
    // Regression test caught when testing the typescript nullifier tree implementation
    DummyComposer composer = DummyComposer();
    BaseRollupInputs empty_inputs = dummy_base_rollup_inputs();

    std::vector<fr> initial_values(7, 0);
    for (size_t i = 0; i < 7; i++) {
        initial_values[i] = i + 1;
    }

    std::tuple<BaseRollupInputs, AppendOnlyTreeSnapshot<NT>, AppendOnlyTreeSnapshot<NT>> inputs_and_snapshots =
        utils::generate_nullifier_tree_testing_values(empty_inputs, new_nullifiers, initial_values);
    BaseRollupInputs testing_inputs = std::get<0>(inputs_and_snapshots);
    AppendOnlyTreeSnapshot<NT> nullifier_tree_start_snapshot = std::get<1>(inputs_and_snapshots);
    AppendOnlyTreeSnapshot<NT> nullifier_tree_end_snapshot = std::get<2>(inputs_and_snapshots);

    /**
     * RUN
     */

    // Run the circuit
    BaseOrMergeRollupPublicInputs outputs =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, testing_inputs);

    /**
     * ASSERT
     */
    // Start state
    ASSERT_EQ(outputs.start_nullifier_tree_snapshot, nullifier_tree_start_snapshot);

    // End state
    ASSERT_EQ(outputs.end_nullifier_tree_snapshot, nullifier_tree_end_snapshot);
}

// Another regression test with values from a failing packages test
TEST_F(base_rollup_tests, nullifier_tree_regression_2)
{
    // Regression test caught when testing the typescript nullifier tree implementation
    std::array<fr, KERNEL_NEW_NULLIFIERS_LENGTH* 2> new_nullifiers = { 0 };
    new_nullifiers[0] = uint256_t("2a7d956c1365d259646d2d85babe1abb793bb8789e98df7e2336a29a0c91fd01");
    new_nullifiers[1] = uint256_t("236bf2d113f9ffee89df1a7a04890c9ad3583c6773eb9cdec484184f66abd4c6");
    new_nullifiers[4] = uint256_t("2f5c8a1ee33c7104b244e22a3e481637cd501c9eae868cfab6b16e3b4ef3d635");
    new_nullifiers[5] = uint256_t("0c484a20780e31747cf9f4f6803986525ed98ef587f5155a1c50689c2cad10ae");

    perform_standard_nullifier_test(new_nullifiers);
}

TEST_F(base_rollup_tests, nullifier_tree_regression_3)
{
    std::array<fr, KERNEL_NEW_NULLIFIERS_LENGTH* 2> new_nullifiers = { 0 };
    new_nullifiers[0] = uint256_t("0740a17aa6437e71836d2adcdcb3f52879bb869cdd9c8fb8dc39a12846cd17f2");
    new_nullifiers[1] = uint256_t("282e0e2f38310a7c7c98b636830b66f3276294560e26ef2499da10892f00af8f");
    new_nullifiers[4] = uint256_t("0f117936e888bd3befb4435f4d65300d25609e95a3d1563f62ef7e58c294f578");
    new_nullifiers[5] = uint256_t("0fcb3908cb15ebf8bab276f5df17524d3b676c8655234e4350953c387fffcdd7");

    perform_standard_nullifier_test(new_nullifiers);
}

// Note leaving this test here as there are no negative tests, even though it no longer passes
TEST_F(base_rollup_tests, native_new_nullifier_tree_sparse_attack)
{
    // @todo THIS SHOULD NOT BE PASSING. The circuit should fail with an assert as we are trying to double-spend.
    /**
     * DESCRIPTION
     */

    DummyComposer composer = DummyComposer();
    BaseRollupInputs empty_inputs = dummy_base_rollup_inputs();

    std::array<fr, KERNEL_NEW_NULLIFIERS_LENGTH* 2> new_nullifiers = { 11, 0, 11, 0, 0, 0, 0, 0 };
    std::tuple<BaseRollupInputs, AppendOnlyTreeSnapshot<NT>, AppendOnlyTreeSnapshot<NT>> inputs_and_snapshots =
        utils::generate_nullifier_tree_testing_values(empty_inputs, new_nullifiers, 1);
    BaseRollupInputs testing_inputs = std::get<0>(inputs_and_snapshots);

    // Run the circuit (SHOULD FAIL WITH AN ASSERT INSTEAD OF THIS!)
    BaseOrMergeRollupPublicInputs outputs =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, testing_inputs);

    EXPECT_EQ(composer.has_failed(), true);
}

TEST_F(base_rollup_tests, native_empty_block_calldata_hash)
{
    DummyComposer composer = DummyComposer();
    // calldata_hash should be computed from leafs of 704 0 bytes. (0x00)
    std::vector<uint8_t> zero_bytes_vec(704, 0);
    auto hash = sha256::sha256(zero_bytes_vec);
    BaseRollupInputs inputs = dummy_base_rollup_inputs();
    BaseOrMergeRollupPublicInputs outputs =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, inputs);

    std::array<fr, 2> calldata_hash_fr = outputs.calldata_hash;
    auto high_buffer = calldata_hash_fr[0].to_buffer();
    auto low_buffer = calldata_hash_fr[1].to_buffer();

    std::array<uint8_t, 32> calldata_hash;
    for (uint8_t i = 0; i < 16; ++i) {
        calldata_hash[i] = high_buffer[16 + i];
        calldata_hash[16 + i] = low_buffer[16 + i];
    }

    ASSERT_EQ(hash, calldata_hash);

    run_cbind(inputs, outputs);
}

TEST_F(base_rollup_tests, native_calldata_hash)
{
    // Execute the base rollup circuit with nullifiers, commitments and a contract deployment. Then check the calldata
    // hash against the expected value.
    DummyComposer composer = DummyComposer();
    BaseRollupInputs inputs = dummy_base_rollup_inputs();
    std::vector<uint8_t> input_data(704, 0);

    // Kernel 1
    // NOTE: nullifier insertions start from 8 as the generate_nullifier_tree_testing_values will populate the every
    // nullifier leaf
    for (uint8_t i = 0; i < 4; ++i) {
        // nullifiers
        input_data[i * 32 + 31] = i + 8; // 8

        // commitments
        input_data[8 * 32 + i * 32 + 31] = i + 1; // 1
        inputs.kernel_data[0].public_inputs.end.new_commitments[i] = fr(i + 1);
    }
    // Kernel 2
    for (uint8_t i = 0; i < 4; ++i) {
        // nullifiers
        input_data[(i + 4) * 32 + 31] = i + 12; // 1

        // commitments
        input_data[8 * 32 + (i + 4) * 32 + 31] = i + 4 + 1; // 1
        inputs.kernel_data[1].public_inputs.end.new_commitments[i] = fr(i + 4 + 1);
    }

    // Get nullifier tree data
    std::tuple<BaseRollupInputs, AppendOnlyTreeSnapshot<NT>, AppendOnlyTreeSnapshot<NT>> inputs_and_snapshots =
        utils::generate_nullifier_tree_testing_values(inputs, 8, 1);
    inputs = std::get<0>(inputs_and_snapshots);

    // Add a contract deployment
    NewContractData<NT> new_contract = {
        .contract_address = fr(1),
        .portal_contract_address = fr(3),
        .function_tree_root = fr(2),
    };
    auto contract_leaf = crypto::pedersen_commitment::compress_native(
        { new_contract.contract_address, new_contract.portal_contract_address, new_contract.function_tree_root },
        GeneratorIndex::CONTRACT_LEAF);
    inputs.kernel_data[0].public_inputs.end.new_contracts[0] = new_contract;
    auto contract_leaf_buffer = contract_leaf.to_buffer();
    auto contract_address_buffer = new_contract.contract_address.to_field().to_buffer();
    auto portal_address_buffer = new_contract.portal_contract_address.to_field().to_buffer();
    for (uint8_t i = 0; i < 32; ++i) {
        input_data[16 * 32 + i] = contract_leaf_buffer[i];
        input_data[18 * 32 + i] = contract_address_buffer[i];
        input_data[20 * 32 + i] = portal_address_buffer[i];
    }

    auto hash = sha256::sha256(input_data);

    BaseOrMergeRollupPublicInputs outputs =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, inputs);

    // Take the two fields and stich them together to get the calldata hash.
    std::array<fr, 2> calldata_hash_fr = outputs.calldata_hash;
    auto high_buffer = calldata_hash_fr[0].to_buffer();
    auto low_buffer = calldata_hash_fr[1].to_buffer();

    std::array<uint8_t, 32> calldata_hash;
    for (uint8_t i = 0; i < 16; ++i) {
        calldata_hash[i] = high_buffer[16 + i];
        calldata_hash[16 + i] = low_buffer[16 + i];
    }

    ASSERT_EQ(hash, calldata_hash);
    run_cbind(inputs, outputs);
}

TEST_F(base_rollup_tests, native_compute_membership_historic_private_data)
{
    // Test membership works for empty trees
    DummyComposer composer = DummyComposer();
    BaseRollupInputs inputs = dummy_base_rollup_inputs();

    auto tree = native_base_rollup::MerkleTree(PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT);
    inputs.constants.start_tree_of_historic_private_data_tree_roots_snapshot = {
        .root = tree.root(),
        .next_available_leaf_index = 0,
    };
    inputs.kernel_data[0]
        .public_inputs.constants.historic_tree_roots.private_historic_tree_roots.private_data_tree_root = fr(0);

    // fetch sibling path from hash path (only get the second half of the hash path)
    auto hash_path = tree.get_hash_path(0);
    std::array<NT::fr, PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT> sibling_path;
    for (size_t i = 0; i < PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT; ++i) {
        sibling_path[i] = hash_path[i].second;
    }
    inputs.historic_private_data_tree_root_membership_witnesses[0] = {
        .leaf_index = 0,
        .sibling_path = sibling_path,
    };

    BaseOrMergeRollupPublicInputs outputs =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, inputs);
}

TEST_F(base_rollup_tests, native_constants_dont_change)
{
    DummyComposer composer = DummyComposer();
    BaseRollupInputs inputs = dummy_base_rollup_inputs();
    BaseOrMergeRollupPublicInputs outputs =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, inputs);
    ASSERT_EQ(inputs.constants, outputs.constants);
    run_cbind(inputs, outputs);
}

TEST_F(base_rollup_tests, native_aggregate)
{
    // TODO: Fix this when aggregation works
    DummyComposer composer = DummyComposer();
    BaseRollupInputs inputs = dummy_base_rollup_inputs();
    BaseOrMergeRollupPublicInputs outputs =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, inputs);
    ASSERT_EQ(inputs.kernel_data[0].public_inputs.end.aggregation_object.public_inputs,
              outputs.end_aggregation_object.public_inputs);
}

TEST_F(base_rollup_tests, native_subtree_height_is_0)
{
    DummyComposer composer = DummyComposer();
    BaseRollupInputs inputs = dummy_base_rollup_inputs();
    BaseOrMergeRollupPublicInputs outputs =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, inputs);
    ASSERT_EQ(outputs.rollup_subtree_height, fr(0));
}

TEST_F(base_rollup_tests, native_proof_verification) {}

TEST_F(base_rollup_tests, native_cbind_0)
{
    BaseRollupInputs inputs = dummy_base_rollup_inputs();
    BaseOrMergeRollupPublicInputs ignored_public_inputs;
    run_cbind(inputs, ignored_public_inputs, false);
}

} // namespace aztec3::circuits::rollup::base::native_base_rollup_circuit
