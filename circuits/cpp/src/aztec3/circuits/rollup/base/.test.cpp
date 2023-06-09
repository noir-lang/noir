#include "c_bind.h"
#include "index.hpp"
#include "init.hpp"

#include "aztec3/circuits/abis/append_only_tree_snapshot.hpp"
#include "aztec3/circuits/abis/combined_accumulated_data.hpp"
#include "aztec3/circuits/abis/kernel_circuit_public_inputs.hpp"
#include "aztec3/circuits/abis/membership_witness.hpp"
#include "aztec3/circuits/abis/new_contract_data.hpp"
#include "aztec3/circuits/abis/previous_kernel_data.hpp"
#include "aztec3/circuits/abis/public_data_read.hpp"
#include "aztec3/circuits/kernel/private/utils.hpp"
#include "aztec3/circuits/rollup/components/components.hpp"
#include "aztec3/circuits/rollup/test_utils/utils.hpp"
#include "aztec3/constants.hpp"

#include <barretenberg/barretenberg.hpp>

#include <gtest/gtest.h>

#include <cstddef>
#include <cstdint>
#include <iostream>
#include <tuple>
#include <vector>


namespace {


using aztec3::circuits::abis::PreviousKernelData;


// using aztec3::circuits::mock::mock_circuit;
using aztec3::circuits::rollup::test_utils::utils::base_rollup_inputs_from_kernels;
using aztec3::circuits::rollup::test_utils::utils::get_empty_kernel;
using aztec3::circuits::rollup::test_utils::utils::get_initial_nullifier_tree;
// using aztec3::circuits::mock::mock_kernel_inputs;

using aztec3::circuits::abis::AppendOnlyTreeSnapshot;

using aztec3::circuits::rollup::native_base_rollup::BaseOrMergeRollupPublicInputs;
using aztec3::circuits::rollup::native_base_rollup::BaseRollupInputs;
using aztec3::circuits::rollup::native_base_rollup::ConstantRollupData;
using aztec3::circuits::rollup::native_base_rollup::NT;

using aztec3::circuits::abis::NewContractData;

using aztec3::circuits::rollup::test_utils::utils::make_public_data_update_request;
using aztec3::circuits::rollup::test_utils::utils::make_public_read;

using DummyComposer = aztec3::utils::DummyComposer;
}  // namespace

namespace aztec3::circuits::rollup::base::native_base_rollup_circuit {

class base_rollup_tests : public ::testing::Test {
  protected:
    static void SetUpTestSuite() { barretenberg::srs::init_crs_factory("../barretenberg/cpp/srs_db/ignition"); }

    static void run_cbind(BaseRollupInputs& base_rollup_inputs,
                          BaseOrMergeRollupPublicInputs& expected_public_inputs,
                          bool compare_pubins = true,
                          bool assert_no_circuit_failure = true)
    {
        info("Retesting via cbinds....");
        // TODO(banks12) might be able to get rid of proving key buffer
        uint8_t const* pk_buf = nullptr;
        size_t const pk_size = base_rollup__init_proving_key(&pk_buf);
        (void)pk_size;
        // info("Proving key size: ", pk_size);

        // TODO(banks12) might be able to get rid of verification key buffer
        uint8_t const* vk_buf = nullptr;
        size_t const vk_size = base_rollup__init_verification_key(pk_buf, &vk_buf);
        (void)vk_size;
        // info("Verification key size: ", vk_size);

        std::vector<uint8_t> base_rollup_inputs_vec;
        write(base_rollup_inputs_vec, base_rollup_inputs);

        // uint8_t const* proof_data;
        // size_t proof_data_size;
        uint8_t const* public_inputs_buf = nullptr;
        size_t public_inputs_size = 0;
        // info("simulating circuit via cbind");
        uint8_t* const circuit_failure_ptr =
            base_rollup__sim(base_rollup_inputs_vec.data(), &public_inputs_size, &public_inputs_buf);

        ASSERT_TRUE(assert_no_circuit_failure ? circuit_failure_ptr == nullptr : circuit_failure_ptr != nullptr);
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

TEST_F(base_rollup_tests, native_no_new_contract_leafs)
{
    DummyComposer composer = DummyComposer("base_rollup_tests__native_no_new_contract_leafs");
    // When there are no contract deployments. The contract tree should be inserting 0 leafs, (not empty leafs);
    // Initially, the start_contract_tree_snapshot is empty (leaf is 0. hash it up).
    // Get sibling path of index 0 leaf (for circuit to check membership via sibling path)
    // No contract leaves -> will insert empty tree -> i.e. end_contract_tree_root = start_contract_tree_root

    BaseRollupInputs emptyInputs = base_rollup_inputs_from_kernels({ get_empty_kernel(), get_empty_kernel() });
    auto empty_contract_tree = native_base_rollup::MerkleTree(CONTRACT_TREE_HEIGHT);

    BaseOrMergeRollupPublicInputs outputs =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, emptyInputs);

    AppendOnlyTreeSnapshot<NT> const expectedStartContractTreeSnapshot = {
        .root = empty_contract_tree.root(),
        .next_available_leaf_index = 0,
    };
    AppendOnlyTreeSnapshot<NT> const expectedEndContractTreeSnapshot = {
        .root = empty_contract_tree.root(),
        .next_available_leaf_index = 2,
    };
    ASSERT_EQ(outputs.start_contract_tree_snapshot, expectedStartContractTreeSnapshot);
    ASSERT_EQ(outputs.end_contract_tree_snapshot, expectedEndContractTreeSnapshot);
    ASSERT_EQ(outputs.start_contract_tree_snapshot, emptyInputs.start_contract_tree_snapshot);
    ASSERT_FALSE(composer.failed());
    run_cbind(emptyInputs, outputs);
}

TEST_F(base_rollup_tests, native_contract_leaf_inserted)
{
    DummyComposer composer = DummyComposer("base_rollup_tests__native_contract_leaf_inserted");
    // When there is a contract deployment, the contract tree should be inserting 1 leaf.
    // The remaining leafs should be 0 leafs, (not empty leafs);

    // Create a "mock" contract deployment
    NewContractData<NT> const new_contract = {
        .contract_address = fr(1),
        .portal_contract_address = fr(3),
        .function_tree_root = fr(2),
    };

    auto empty_contract_tree = native_base_rollup::MerkleTree(CONTRACT_TREE_HEIGHT);
    AppendOnlyTreeSnapshot<NT> const expected_start_contracts_snapshot = {
        .root = empty_contract_tree.root(),
        .next_available_leaf_index = 0,
    };

    // create expected end contract tree snapshot
    auto expected_end_contracts_snapshot_tree = stdlib::merkle_tree::MemoryTree(CONTRACT_TREE_HEIGHT);
    expected_end_contracts_snapshot_tree.update_element(0, new_contract.hash());

    AppendOnlyTreeSnapshot<NT> const expected_end_contracts_snapshot = {
        .root = expected_end_contracts_snapshot_tree.root(),
        .next_available_leaf_index = 2,
    };

    std::array<PreviousKernelData<NT>, 2> kernel_data = { get_empty_kernel(), get_empty_kernel() };
    kernel_data[0].public_inputs.end.new_contracts[0] = new_contract;
    BaseRollupInputs inputs = base_rollup_inputs_from_kernels(kernel_data);

    BaseOrMergeRollupPublicInputs outputs =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, inputs);

    ASSERT_EQ(outputs.start_contract_tree_snapshot, expected_start_contracts_snapshot);
    ASSERT_EQ(outputs.start_contract_tree_snapshot, inputs.start_contract_tree_snapshot);
    ASSERT_EQ(outputs.end_contract_tree_snapshot, expected_end_contracts_snapshot);
    ASSERT_FALSE(composer.failed());
    run_cbind(inputs, outputs);
}

TEST_F(base_rollup_tests, native_contract_leaf_inserted_in_non_empty_snapshot_tree)
{
    DummyComposer composer =
        DummyComposer("base_rollup_tests__native_contract_leaf_inserted_in_non_empty_snapshot_tree");
    // Same as before except our start_contract_snapshot_tree is not empty
    std::array<PreviousKernelData<NT>, 2> kernel_data = { get_empty_kernel(), get_empty_kernel() };

    // Create a "mock" contract deployment
    NewContractData<NT> new_contract = {
        .contract_address = fr(1),
        .portal_contract_address = fr(3),
        .function_tree_root = fr(2),
    };
    kernel_data[0].public_inputs.end.new_contracts[0] = new_contract;
    BaseRollupInputs inputs = base_rollup_inputs_from_kernels(kernel_data);

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
    auto sibling_path = test_utils::utils::get_sibling_path<CONTRACT_SUBTREE_INCLUSION_CHECK_DEPTH>(
        start_contract_tree_snapshot, 12, CONTRACT_SUBTREE_DEPTH);
    inputs.new_contracts_subtree_sibling_path = sibling_path;

    // create expected end contract tree snapshot
    auto expected_contract_leaf = crypto::pedersen_commitment::compress_native(
        { new_contract.contract_address, new_contract.portal_contract_address, new_contract.function_tree_root },
        GeneratorIndex::CONTRACT_LEAF);
    auto expected_end_contracts_snapshot_tree = start_contract_tree_snapshot;
    expected_end_contracts_snapshot_tree.update_element(12, expected_contract_leaf);

    AppendOnlyTreeSnapshot<NT> const expected_end_contracts_snapshot = {
        .root = expected_end_contracts_snapshot_tree.root(),
        .next_available_leaf_index = 14,
    };
    BaseOrMergeRollupPublicInputs outputs =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, inputs);

    ASSERT_EQ(outputs.start_contract_tree_snapshot, inputs.start_contract_tree_snapshot);
    ASSERT_EQ(outputs.end_contract_tree_snapshot, expected_end_contracts_snapshot);
    ASSERT_FALSE(composer.failed());
    run_cbind(inputs, outputs);
}

TEST_F(base_rollup_tests, native_new_commitments_tree)
{
    DummyComposer composer = DummyComposer("base_rollup_tests__native_new_commitments_tree");
    // Create 4 new mock commitments. Add them to kernel data.
    // Then get sibling path so we can verify insert them into the tree.

    std::array<PreviousKernelData<NT>, 2> kernel_data = { get_empty_kernel(), get_empty_kernel() };
    std::array<NT::fr, KERNEL_NEW_COMMITMENTS_LENGTH* 2> new_commitments = { 0, 1, 2, 3, 4, 5, 6, 7 };
    for (uint8_t i = 0; i < 2; i++) {
        std::array<fr, KERNEL_NEW_COMMITMENTS_LENGTH> kernel_commitments;
        for (uint8_t j = 0; j < KERNEL_NEW_COMMITMENTS_LENGTH; j++) {
            kernel_commitments[j] = new_commitments[i * KERNEL_NEW_COMMITMENTS_LENGTH + j];
        }
        kernel_data[i].public_inputs.end.new_commitments = kernel_commitments;
    }

    // get sibling path
    auto private_data_tree = native_base_rollup::MerkleTree(PRIVATE_DATA_TREE_HEIGHT);
    AppendOnlyTreeSnapshot<NT> const expected_start_commitments_snapshot = {
        .root = private_data_tree.root(),
        .next_available_leaf_index = 0,
    };
    for (size_t i = 0; i < new_commitments.size(); ++i) {
        private_data_tree.update_element(i, new_commitments[i]);
    }
    AppendOnlyTreeSnapshot<NT> const expected_end_commitments_snapshot = {
        .root = private_data_tree.root(),
        .next_available_leaf_index = 8,
    };

    auto inputs = base_rollup_inputs_from_kernels(kernel_data);
    BaseOrMergeRollupPublicInputs outputs =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, inputs);

    ASSERT_EQ(outputs.start_private_data_tree_snapshot, expected_start_commitments_snapshot);
    ASSERT_EQ(outputs.start_private_data_tree_snapshot, inputs.start_private_data_tree_snapshot);
    ASSERT_EQ(outputs.end_private_data_tree_snapshot, expected_end_commitments_snapshot);
    ASSERT_FALSE(composer.failed());
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

    std::array<fr, KERNEL_NEW_NULLIFIERS_LENGTH* 2> const new_nullifiers = { 0, 0, 0, 0, 0, 0, 0, 0 };
    auto nullifier_tree = get_initial_nullifier_tree({ 1, 2, 3, 4, 5, 6, 7 });
    auto start_nullifier_tree_snapshot = nullifier_tree.get_snapshot();
    for (auto v : new_nullifiers) {
        nullifier_tree.update_element(v);
    }
    auto end_nullifier_tree_snapshot = nullifier_tree.get_snapshot();

    /**
     * RUN
     */
    DummyComposer composer = DummyComposer("base_rollup_tests__native_new_nullifier_tree_empty");
    std::array<PreviousKernelData<NT>, 2> const kernel_data = { get_empty_kernel(), get_empty_kernel() };
    BaseRollupInputs const empty_inputs = base_rollup_inputs_from_kernels(kernel_data);

    BaseOrMergeRollupPublicInputs const outputs =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, empty_inputs);

    /**
     * ASSERT
     */
    // Start state
    ASSERT_EQ(outputs.start_nullifier_tree_snapshot, start_nullifier_tree_snapshot);

    // End state
    ASSERT_EQ(outputs.end_nullifier_tree_snapshot, end_nullifier_tree_snapshot);
    ASSERT_EQ(outputs.end_nullifier_tree_snapshot.root, outputs.start_nullifier_tree_snapshot.root);
    ASSERT_EQ(outputs.end_nullifier_tree_snapshot.next_available_leaf_index,
              outputs.start_nullifier_tree_snapshot.next_available_leaf_index + 8);
    ASSERT_FALSE(composer.failed());
}

void nullifier_insertion_test(std::array<fr, KERNEL_NEW_NULLIFIERS_LENGTH * 2> new_nullifiers)
{
    // @todo We can probably reuse this more than we are already doing.
    // Regression test caught when testing the typescript nullifier tree implementation

    auto nullifier_tree = get_initial_nullifier_tree({ 1, 2, 3, 4, 5, 6, 7 });
    auto start_nullifier_tree_snapshot = nullifier_tree.get_snapshot();
    for (auto v : new_nullifiers) {
        nullifier_tree.update_element(v);
    }
    auto end_nullifier_tree_snapshot = nullifier_tree.get_snapshot();

    DummyComposer composer = DummyComposer("base_rollup_tests__nullifier_insertion_test");
    std::array<PreviousKernelData<NT>, 2> kernel_data = { get_empty_kernel(), get_empty_kernel() };
    for (uint8_t i = 0; i < 2; i++) {
        std::array<fr, KERNEL_NEW_NULLIFIERS_LENGTH> kernel_nullifiers;
        for (uint8_t j = 0; j < KERNEL_NEW_NULLIFIERS_LENGTH; j++) {
            kernel_nullifiers[j] = new_nullifiers[i * KERNEL_NEW_NULLIFIERS_LENGTH + j];
        }
        kernel_data[i].public_inputs.end.new_nullifiers = kernel_nullifiers;
    }
    BaseRollupInputs const inputs = base_rollup_inputs_from_kernels(kernel_data);

    BaseOrMergeRollupPublicInputs const outputs =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, inputs);
    /**
     * ASSERT
     */
    ASSERT_EQ(outputs.start_nullifier_tree_snapshot, start_nullifier_tree_snapshot);
    ASSERT_EQ(outputs.end_nullifier_tree_snapshot, end_nullifier_tree_snapshot);
    ASSERT_EQ(outputs.end_nullifier_tree_snapshot.next_available_leaf_index,
              outputs.start_nullifier_tree_snapshot.next_available_leaf_index + KERNEL_NEW_NULLIFIERS_LENGTH * 2);
    ASSERT_FALSE(composer.failed());
}

TEST_F(base_rollup_tests, native_new_nullifier_tree_all_larger)
{
    nullifier_insertion_test({ 8, 9, 10, 11, 12, 13, 14, 15 });
}

TEST_F(base_rollup_tests, native_new_nullifier_tree_sparse_insertions)
{
    nullifier_insertion_test({ 9, 11, 16, 21, 26, 31, 36, 41 });
}

TEST_F(base_rollup_tests, native_new_nullifier_tree_sparse)
{
    /**
     * DESCRIPTION
     */
    std::vector<fr> const initial_values = { 5, 10, 15, 20, 25, 30, 35 };
    std::array<fr, KERNEL_NEW_NULLIFIERS_LENGTH* 2> const nullifiers = { 6, 11, 16, 21, 26, 31, 36, 41 };

    auto nullifier_tree = get_initial_nullifier_tree(initial_values);
    auto expected_start_nullifier_tree_snapshot = nullifier_tree.get_snapshot();
    for (auto v : nullifiers) {
        nullifier_tree.update_element(v);
    }
    auto expected_end_nullifier_tree_snapshot = nullifier_tree.get_snapshot();

    DummyComposer composer = DummyComposer("base_rollup_tests__native_new_nullifier_tree_sparse");
    BaseRollupInputs const empty_inputs = base_rollup_inputs_from_kernels({ get_empty_kernel(), get_empty_kernel() });
    std::tuple<BaseRollupInputs, AppendOnlyTreeSnapshot<NT>, AppendOnlyTreeSnapshot<NT>> inputs_and_snapshots =
        test_utils::utils::generate_nullifier_tree_testing_values_explicit(empty_inputs, nullifiers, initial_values);

    BaseRollupInputs const testing_inputs = std::get<0>(inputs_and_snapshots);

    /**
     * RUN
     */

    // Run the circuit
    BaseOrMergeRollupPublicInputs const outputs =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, testing_inputs);

    /**
     * ASSERT
     */
    // Start state
    ASSERT_EQ(outputs.start_nullifier_tree_snapshot, expected_start_nullifier_tree_snapshot);

    // End state
    ASSERT_EQ(outputs.end_nullifier_tree_snapshot, expected_end_nullifier_tree_snapshot);
    ASSERT_FALSE(composer.failed());
}

TEST_F(base_rollup_tests, native_nullifier_tree_regression)
{
    // Regression test caught when testing the typescript nullifier tree implementation
    DummyComposer composer = DummyComposer("base_rollup_tests__native_nullifier_tree_regression");

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

    auto nullifier_tree = get_initial_nullifier_tree(initial_values);
    auto expected_start_nullifier_tree_snapshot = nullifier_tree.get_snapshot();
    for (auto v : new_nullifiers) {
        nullifier_tree.update_element(v);
    }
    auto expected_end_nullifier_tree_snapshot = nullifier_tree.get_snapshot();

    /**
     * RUN
     */
    BaseRollupInputs const empty_inputs = base_rollup_inputs_from_kernels({ get_empty_kernel(), get_empty_kernel() });
    std::tuple<BaseRollupInputs, AppendOnlyTreeSnapshot<NT>, AppendOnlyTreeSnapshot<NT>> inputs_and_snapshots =
        test_utils::utils::generate_nullifier_tree_testing_values_explicit(
            empty_inputs, new_nullifiers, initial_values);
    BaseRollupInputs const testing_inputs = std::get<0>(inputs_and_snapshots);
    // Run the circuit
    BaseOrMergeRollupPublicInputs const outputs =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, testing_inputs);

    /**
     * ASSERT
     */
    // Start state
    ASSERT_EQ(outputs.start_nullifier_tree_snapshot, expected_start_nullifier_tree_snapshot);

    // End state
    ASSERT_EQ(outputs.end_nullifier_tree_snapshot, expected_end_nullifier_tree_snapshot);
    ASSERT_FALSE(composer.failed());
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

    nullifier_insertion_test(new_nullifiers);
}

TEST_F(base_rollup_tests, nullifier_tree_regression_3)
{
    std::array<fr, KERNEL_NEW_NULLIFIERS_LENGTH* 2> new_nullifiers = { 0 };
    new_nullifiers[0] = uint256_t("0740a17aa6437e71836d2adcdcb3f52879bb869cdd9c8fb8dc39a12846cd17f2");
    new_nullifiers[1] = uint256_t("282e0e2f38310a7c7c98b636830b66f3276294560e26ef2499da10892f00af8f");
    new_nullifiers[4] = uint256_t("0f117936e888bd3befb4435f4d65300d25609e95a3d1563f62ef7e58c294f578");
    new_nullifiers[5] = uint256_t("0fcb3908cb15ebf8bab276f5df17524d3b676c8655234e4350953c387fffcdd7");

    nullifier_insertion_test(new_nullifiers);
}

TEST_F(base_rollup_tests, native_new_nullifier_tree_double_spend)
{
    /**
     * DESCRIPTION
     */

    DummyComposer composer = DummyComposer("base_rollup_tests__native_new_nullifier_tree_double_spend");
    BaseRollupInputs const empty_inputs = base_rollup_inputs_from_kernels({ get_empty_kernel(), get_empty_kernel() });

    std::array<fr, KERNEL_NEW_NULLIFIERS_LENGTH* 2> const new_nullifiers = { 11, 0, 11, 0, 0, 0, 0, 0 };
    std::tuple<BaseRollupInputs, AppendOnlyTreeSnapshot<NT>, AppendOnlyTreeSnapshot<NT>> inputs_and_snapshots =
        test_utils::utils::generate_nullifier_tree_testing_values(empty_inputs, new_nullifiers, 1);
    BaseRollupInputs const testing_inputs = std::get<0>(inputs_and_snapshots);

    BaseOrMergeRollupPublicInputs const outputs =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, testing_inputs);

    ASSERT_TRUE(composer.failed());
    ASSERT_EQ(composer.get_first_failure().message, "Nullifier is not in the correct range");
}

TEST_F(base_rollup_tests, native_empty_block_calldata_hash)
{
    DummyComposer composer = DummyComposer("base_rollup_tests__native_empty_block_calldata_hash");
    std::vector<uint8_t> const zero_bytes_vec = test_utils::utils::get_empty_calldata_leaf();
    auto hash = sha256::sha256(zero_bytes_vec);
    BaseRollupInputs inputs = base_rollup_inputs_from_kernels({ get_empty_kernel(), get_empty_kernel() });
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
    ASSERT_FALSE(composer.failed());

    run_cbind(inputs, outputs);
}

TEST_F(base_rollup_tests, native_calldata_hash)
{
    // Execute the base rollup circuit with nullifiers, commitments and a contract deployment. Then check the calldata
    // hash against the expected value.
    std::array<PreviousKernelData<NT>, 2> kernel_data = { get_empty_kernel(), get_empty_kernel() };

    // Commitments inserted are [1,2,3,4,5,6,7,8]. Nullifiers inserted are [8,9,10,11,12,13,14,15]
    for (size_t i = 0; i < 2; ++i) {
        for (size_t j = 0; j < 4; j++) {
            kernel_data[i].public_inputs.end.new_commitments[j] = fr(i * 4 + j + 1);
            kernel_data[i].public_inputs.end.new_nullifiers[j] = fr(i * 4 + j + 8);
        }
    }

    // Add a contract deployment
    NewContractData<NT> const new_contract = {
        .contract_address = fr(1),
        .portal_contract_address = fr(3),
        .function_tree_root = fr(2),
    };
    kernel_data[0].public_inputs.end.new_contracts[0] = new_contract;

    std::array<fr, 2> const expected_hash = components::compute_kernels_calldata_hash(kernel_data);

    DummyComposer composer = DummyComposer("base_rollup_tests__native_calldata_hash");
    BaseRollupInputs inputs = base_rollup_inputs_from_kernels(kernel_data);
    BaseOrMergeRollupPublicInputs outputs =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, inputs);

    // Take the two fields and stich them together to get the calldata hash.
    std::array<fr, 2> const calldata_hash_fr = outputs.calldata_hash;

    ASSERT_EQ(expected_hash, calldata_hash_fr);

    ASSERT_FALSE(composer.failed());
    run_cbind(inputs, outputs);
}

TEST_F(base_rollup_tests, native_compute_membership_historic_private_data_negative)
{
    // WRITE a negative test that will fail the inclusion proof

    // Test membership works for empty trees
    DummyComposer composer =
        DummyComposer("base_rollup_tests__native_compute_membership_historic_private_data_negative");
    std::array<PreviousKernelData<NT>, 2> const kernel_data = { get_empty_kernel(), get_empty_kernel() };
    BaseRollupInputs inputs = base_rollup_inputs_from_kernels(kernel_data);

    auto private_data_tree = native_base_rollup::MerkleTree(PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT);

    // Create an INCORRECT sibling path for the private data tree root in the historic tree roots.
    auto hash_path = private_data_tree.get_sibling_path(0);
    std::array<NT::fr, PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT> sibling_path;
    for (size_t i = 0; i < PRIVATE_DATA_TREE_ROOTS_TREE_HEIGHT; ++i) {
        sibling_path[i] = hash_path[i] + 1;
    }
    inputs.historic_private_data_tree_root_membership_witnesses[0] = {
        .leaf_index = 0,
        .sibling_path = sibling_path,
    };

    BaseOrMergeRollupPublicInputs const outputs =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, inputs);

    ASSERT_TRUE(composer.failed());
    ASSERT_EQ(composer.get_first_failure().message, "Membership check failed: historic private data tree roots 0");
}

TEST_F(base_rollup_tests, native_compute_membership_historic_contract_tree_negative)
{
    // Test membership works for empty trees
    DummyComposer composer =
        DummyComposer("base_rollup_tests__native_compute_membership_historic_contract_tree_negative");
    std::array<PreviousKernelData<NT>, 2> const kernel_data = { get_empty_kernel(), get_empty_kernel() };
    BaseRollupInputs inputs = base_rollup_inputs_from_kernels(kernel_data);

    auto contract_tree = native_base_rollup::MerkleTree(CONTRACT_TREE_ROOTS_TREE_HEIGHT);

    // Create an INCORRECT sibling path for contract tree root in the historic tree roots.
    auto hash_path = contract_tree.get_sibling_path(0);
    std::array<NT::fr, CONTRACT_TREE_ROOTS_TREE_HEIGHT> sibling_path;
    for (size_t i = 0; i < CONTRACT_TREE_ROOTS_TREE_HEIGHT; ++i) {
        sibling_path[i] = hash_path[i] + 1;
    }
    inputs.historic_contract_tree_root_membership_witnesses[0] = {
        .leaf_index = 0,
        .sibling_path = sibling_path,
    };

    BaseOrMergeRollupPublicInputs const outputs =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, inputs);

    ASSERT_TRUE(composer.failed());
    ASSERT_EQ(composer.get_first_failure().message, "Membership check failed: historic contract data tree roots 0");
}

TEST_F(base_rollup_tests, native_constants_dont_change)
{
    DummyComposer composer = DummyComposer("base_rollup_tests__native_constants_dont_change");
    BaseRollupInputs inputs = base_rollup_inputs_from_kernels({ get_empty_kernel(), get_empty_kernel() });
    BaseOrMergeRollupPublicInputs outputs =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, inputs);
    ASSERT_EQ(inputs.constants, outputs.constants);
    EXPECT_FALSE(composer.failed());
    run_cbind(inputs, outputs);
}

TEST_F(base_rollup_tests, native_aggregate)
{
    // TODO(rahul): Fix this when aggregation works
    DummyComposer composer = DummyComposer("base_rollup_tests__native_aggregate");
    BaseRollupInputs inputs = base_rollup_inputs_from_kernels({ get_empty_kernel(), get_empty_kernel() });
    BaseOrMergeRollupPublicInputs const outputs =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, inputs);
    ASSERT_EQ(inputs.kernel_data[0].public_inputs.end.aggregation_object.public_inputs,
              outputs.end_aggregation_object.public_inputs);
    ASSERT_FALSE(composer.failed());
}

TEST_F(base_rollup_tests, native_subtree_height_is_0)
{
    DummyComposer composer = DummyComposer("base_rollup_tests__native_subtree_height_is_0");
    BaseRollupInputs const inputs = base_rollup_inputs_from_kernels({ get_empty_kernel(), get_empty_kernel() });
    BaseOrMergeRollupPublicInputs const outputs =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, inputs);
    ASSERT_EQ(outputs.rollup_subtree_height, fr(0));
    ASSERT_FALSE(composer.failed());
}

TEST_F(base_rollup_tests, native_cbind_0)
{
    // @todo Error handling?
    BaseRollupInputs inputs = base_rollup_inputs_from_kernels({ get_empty_kernel(), get_empty_kernel() });
    BaseOrMergeRollupPublicInputs ignored_public_inputs;
    run_cbind(inputs, ignored_public_inputs, false);
}

TEST_F(base_rollup_tests, native_single_public_state_read)
{
    DummyComposer composer = DummyComposer("base_rollup_tests__native_single_public_state_read");
    native_base_rollup::MerkleTree private_data_tree(PRIVATE_DATA_TREE_HEIGHT);
    native_base_rollup::MerkleTree contract_tree(CONTRACT_TREE_HEIGHT);
    stdlib::merkle_tree::MemoryStore public_data_tree_store;
    native_base_rollup::SparseTree public_data_tree(public_data_tree_store, PUBLIC_DATA_TREE_HEIGHT);
    native_base_rollup::MerkleTree l1_to_l2_messages_tree(L1_TO_L2_MSG_TREE_HEIGHT);

    auto data_read = abis::PublicDataRead<NT>{
        .leaf_index = fr(1),
        .value = fr(42),
    };

    std::array<PreviousKernelData<NT>, 2> kernel_data = { get_empty_kernel(), get_empty_kernel() };
    kernel_data[0].public_inputs.end.public_data_reads[0] = data_read;
    auto inputs = test_utils::utils::base_rollup_inputs_from_kernels(
        kernel_data, private_data_tree, contract_tree, public_data_tree, l1_to_l2_messages_tree);

    BaseOrMergeRollupPublicInputs outputs =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, inputs);

    ASSERT_EQ(outputs.start_public_data_tree_root, inputs.start_public_data_tree_root);
    ASSERT_EQ(outputs.end_public_data_tree_root, public_data_tree.root());
    ASSERT_EQ(outputs.end_public_data_tree_root, outputs.start_public_data_tree_root);
    ASSERT_FALSE(composer.failed());
    run_cbind(inputs, outputs);
}

TEST_F(base_rollup_tests, native_single_public_state_write)
{
    DummyComposer composer = DummyComposer("base_rollup_tests__native_single_public_state_write");
    native_base_rollup::MerkleTree private_data_tree(PRIVATE_DATA_TREE_HEIGHT);
    native_base_rollup::MerkleTree contract_tree(CONTRACT_TREE_HEIGHT);
    stdlib::merkle_tree::MemoryStore public_data_tree_store;
    native_base_rollup::SparseTree public_data_tree(public_data_tree_store, PUBLIC_DATA_TREE_HEIGHT);
    native_base_rollup::MerkleTree l1_to_l2_messages_tree(L1_TO_L2_MSG_TREE_HEIGHT);

    auto data_write = abis::PublicDataUpdateRequest<NT>{
        .leaf_index = fr(1),
        .old_value = fr(2),
        .new_value = fr(42),
    };

    std::array<PreviousKernelData<NT>, 2> kernel_data = { get_empty_kernel(), get_empty_kernel() };
    kernel_data[0].public_inputs.end.public_data_update_requests[0] = data_write;

    auto inputs = test_utils::utils::base_rollup_inputs_from_kernels(
        kernel_data, private_data_tree, contract_tree, public_data_tree, l1_to_l2_messages_tree);

    BaseOrMergeRollupPublicInputs outputs =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, inputs);

    ASSERT_EQ(outputs.start_public_data_tree_root, inputs.start_public_data_tree_root);
    ASSERT_EQ(outputs.end_public_data_tree_root, public_data_tree.root());
    ASSERT_NE(outputs.end_public_data_tree_root, outputs.start_public_data_tree_root);
    ASSERT_FALSE(composer.failed());
    run_cbind(inputs, outputs);
}

TEST_F(base_rollup_tests, native_multiple_public_state_read_writes)
{
    DummyComposer composer = DummyComposer("base_rollup_tests__native_multiple_public_state_read_writes");
    native_base_rollup::MerkleTree private_data_tree(PRIVATE_DATA_TREE_HEIGHT);
    native_base_rollup::MerkleTree contract_tree(CONTRACT_TREE_HEIGHT);
    stdlib::merkle_tree::MemoryStore public_data_tree_store;
    native_base_rollup::SparseTree public_data_tree(public_data_tree_store, PUBLIC_DATA_TREE_HEIGHT);
    native_base_rollup::MerkleTree l1_to_l2_messages_tree(L1_TO_L2_MSG_TREE_HEIGHT);

    std::array<PreviousKernelData<NT>, 2> kernel_data = { get_empty_kernel(), get_empty_kernel() };

    // We set up reads and writes such that the right tx will read or write to indices already modified by the left tx
    kernel_data[0].public_inputs.end.public_data_reads[0] = make_public_read(fr(1), fr(101));
    kernel_data[0].public_inputs.end.public_data_reads[1] = make_public_read(fr(2), fr(102));
    kernel_data[0].public_inputs.end.public_data_update_requests[0] =
        make_public_data_update_request(fr(3), fr(103), fr(203));
    kernel_data[0].public_inputs.end.public_data_update_requests[1] =
        make_public_data_update_request(fr(4), fr(104), fr(204));
    kernel_data[0].public_inputs.end.public_data_update_requests[2] =
        make_public_data_update_request(fr(5), fr(105), fr(205));

    kernel_data[1].public_inputs.end.public_data_reads[0] = make_public_read(fr(3), fr(203));
    kernel_data[1].public_inputs.end.public_data_reads[1] = make_public_read(fr(11), fr(211));
    kernel_data[1].public_inputs.end.public_data_update_requests[0] =
        make_public_data_update_request(fr(12), fr(212), fr(312));
    kernel_data[1].public_inputs.end.public_data_update_requests[1] =
        make_public_data_update_request(fr(4), fr(204), fr(304));

    auto inputs = test_utils::utils::base_rollup_inputs_from_kernels(
        kernel_data, private_data_tree, contract_tree, public_data_tree, l1_to_l2_messages_tree);

    BaseOrMergeRollupPublicInputs outputs =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, inputs);

    ASSERT_EQ(outputs.start_public_data_tree_root, inputs.start_public_data_tree_root);
    ASSERT_EQ(outputs.end_public_data_tree_root, public_data_tree.root());
    ASSERT_NE(outputs.end_public_data_tree_root, outputs.start_public_data_tree_root);
    ASSERT_FALSE(composer.failed());
    run_cbind(inputs, outputs);
}

TEST_F(base_rollup_tests, native_invalid_public_state_read)
{
    DummyComposer composer = DummyComposer("base_rollup_tests__native_invalid_public_state_read");
    native_base_rollup::MerkleTree private_data_tree(PRIVATE_DATA_TREE_HEIGHT);
    native_base_rollup::MerkleTree contract_tree(CONTRACT_TREE_HEIGHT);
    stdlib::merkle_tree::MemoryStore public_data_tree_store;
    native_base_rollup::SparseTree public_data_tree(public_data_tree_store, PUBLIC_DATA_TREE_HEIGHT);
    native_base_rollup::MerkleTree l1_to_l2_messages_tree(L1_TO_L2_MSG_TREE_HEIGHT);

    auto data_read = abis::PublicDataRead<NT>{
        .leaf_index = fr(1),
        .value = fr(42),
    };

    std::array<PreviousKernelData<NT>, 2> kernel_data = { get_empty_kernel(), get_empty_kernel() };
    kernel_data[0].public_inputs.end.public_data_reads[0] = data_read;
    auto inputs = test_utils::utils::base_rollup_inputs_from_kernels(
        kernel_data, private_data_tree, contract_tree, public_data_tree, l1_to_l2_messages_tree);

    // We change the initial tree root so the read value does not match
    public_data_tree.update_element(1, fr(43));
    inputs.start_public_data_tree_root = public_data_tree.root();

    BaseOrMergeRollupPublicInputs outputs =
        aztec3::circuits::rollup::native_base_rollup::base_rollup_circuit(composer, inputs);

    ASSERT_EQ(outputs.start_public_data_tree_root, inputs.start_public_data_tree_root);
    ASSERT_EQ(outputs.end_public_data_tree_root, public_data_tree.root());
    ASSERT_EQ(outputs.end_public_data_tree_root, outputs.start_public_data_tree_root);
    ASSERT_TRUE(composer.failed());
    run_cbind(inputs, outputs, true, false);
}

}  // namespace aztec3::circuits::rollup::base::native_base_rollup_circuit
