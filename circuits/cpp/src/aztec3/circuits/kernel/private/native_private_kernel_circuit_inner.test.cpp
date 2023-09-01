#include "c_bind.h"
#include "testing_harness.hpp"

#include "aztec3/circuits/abis/membership_witness.hpp"
#include "aztec3/circuits/apps/test_apps/escrow/deposit.hpp"
#include "aztec3/circuits/kernel/private/common.hpp"
#include "aztec3/constants.hpp"
#include "aztec3/utils/array.hpp"
#include "aztec3/utils/circuit_errors.hpp"

#include <barretenberg/barretenberg.hpp>

#include <gtest/gtest.h>

#include <array>
#include <cstdint>

namespace aztec3::circuits::kernel::private_kernel {

using aztec3::circuits::apps::test_apps::escrow::deposit;

using aztec3::circuits::kernel::private_kernel::testing_harness::do_private_call_get_kernel_inputs_inner;
using aztec3::circuits::kernel::private_kernel::testing_harness::get_random_reads;
using aztec3::circuits::kernel::private_kernel::testing_harness::validate_no_new_deployed_contract;

using aztec3::utils::array_length;
using aztec3::utils::CircuitErrorCode;

// NOTE: *DO NOT* call fr constructors in static initializers and assign them to constants. This will fail. Instead, use
// lazy initialization or functions. Lambdas were introduced here.
// amount = 5,  asset_id = 1, memo = 999
const auto standard_test_args = [] { return std::vector<NT::fr>{ NT::fr(5), NT::fr(1), NT::fr(999) }; };

class native_private_kernel_inner_tests : public ::testing::Test {
  protected:
    static void SetUpTestSuite() { barretenberg::srs::init_crs_factory("../barretenberg/cpp/srs_db/ignition"); }
};

/**
 **************************************************************
 * Native inner private kernel circuit tests.
 **************************************************************
 */
TEST_F(native_private_kernel_inner_tests, private_function_zero_storage_contract_address_fails)
{
    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    // Set storage_contract_address to 0
    private_inputs.private_call.call_stack_item.public_inputs.call_context.storage_contract_address = 0;
    private_inputs.private_call.call_stack_item.contract_address = 0;

    // Modify the call stack item's hash with the newly added contract address.
    private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] =
        private_inputs.private_call.call_stack_item.hash();

    // Invoke the native private kernel circuit
    DummyBuilder builder = DummyBuilder("private_kernel_tests__private_function_zero_storage_contract_address_fails");
    native_private_kernel_circuit_inner(builder, private_inputs);

    // Assertion checks
    EXPECT_TRUE(builder.failed());
    EXPECT_EQ(builder.get_first_failure().code, CircuitErrorCode::PRIVATE_KERNEL__INVALID_CONTRACT_ADDRESS);
    EXPECT_EQ(builder.get_first_failure().message,
              "contract address can't be 0 for non-contract deployment related transactions");
}


TEST_F(native_private_kernel_inner_tests, private_function_incorrect_is_internal)
{
    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    // Make the call internal but msg_sender != storage_contract_address.
    private_inputs.private_call.call_stack_item.function_data.is_internal = true;
    private_inputs.private_call.call_stack_item.public_inputs.call_context.msg_sender = 1;
    private_inputs.private_call.call_stack_item.public_inputs.call_context.storage_contract_address = 2;

    // We need to update the previous_kernel's private_call_stack because the current_call_stack_item has changed
    // i.e. we changed the function_data and public_inputs->call_context of the current_call_stack_item
    private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] =
        private_inputs.private_call.call_stack_item.hash();

    // Invoke the native private kernel circuit
    DummyBuilder builder = DummyBuilder("private_kernel_tests__private_function_incorrect_contract_tree_root_fails");
    native_private_kernel_circuit_inner(builder, private_inputs);

    // Assertion checks
    EXPECT_TRUE(builder.failed());
    EXPECT_EQ(builder.get_first_failure().code, CircuitErrorCode::PRIVATE_KERNEL__IS_INTERNAL_BUT_NOT_SELF_CALL);
    EXPECT_EQ(builder.get_first_failure().message, "call is internal, but msg_sender is not self");
}

TEST_F(native_private_kernel_inner_tests, private_function_incorrect_contract_tree_root_fails)
{
    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    // Set historic_tree_root to a random scalar.
    private_inputs.previous_kernel.public_inputs.constants.block_data.contract_tree_root = NT::fr::random_element();

    // Invoke the native private kernel circuit
    DummyBuilder builder = DummyBuilder("private_kernel_tests__private_function_incorrect_contract_tree_root_fails");
    native_private_kernel_circuit_inner(builder, private_inputs);

    // Assertion checks
    EXPECT_TRUE(builder.failed());
    EXPECT_EQ(
        builder.get_first_failure().code,
        CircuitErrorCode::PRIVATE_KERNEL__PURPORTED_CONTRACT_TREE_ROOT_AND_PREVIOUS_KERNEL_CONTRACT_TREE_ROOT_MISMATCH);
    EXPECT_EQ(builder.get_first_failure().message,
              "purported_contract_tree_root doesn't match previous_kernel_contract_tree_root");
}

TEST_F(native_private_kernel_inner_tests, private_function_incorrect_contract_leaf_index_fails)
{
    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    // Set the leaf index of the contract leaf to 20 (the correct value is 1).
    NT::fr const wrong_idx = 20;
    private_inputs.private_call.contract_leaf_membership_witness.leaf_index = wrong_idx;

    // Invoke the native private kernel circuit
    DummyBuilder builder = DummyBuilder("private_kernel_tests__private_function_incorrect_contract_leaf_index_fails");
    native_private_kernel_circuit_inner(builder, private_inputs);

    // Assertion checks
    EXPECT_TRUE(builder.failed());
    EXPECT_EQ(builder.get_first_failure().code,
              CircuitErrorCode::PRIVATE_KERNEL__COMPUTED_CONTRACT_TREE_ROOT_AND_PURPORTED_CONTRACT_TREE_ROOT_MISMATCH);
    EXPECT_EQ(builder.get_first_failure().message,
              "computed_contract_tree_root doesn't match purported_contract_tree_root");
}

TEST_F(native_private_kernel_inner_tests, private_function_incorrect_contract_leaf_sibling_path_fails)
{
    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    // Change the contract leaf's membership proof.
    private_inputs.private_call.contract_leaf_membership_witness.sibling_path[0] = fr::random_element();

    // Invoke the native private kernel circuit
    DummyBuilder builder =
        DummyBuilder("private_kernel_tests__private_function_incorrect_contract_leaf_sibling_path_fails");
    native_private_kernel_circuit_inner(builder, private_inputs);

    // Assertion checks
    EXPECT_TRUE(builder.failed());
    EXPECT_EQ(builder.get_first_failure().code,
              CircuitErrorCode::PRIVATE_KERNEL__COMPUTED_CONTRACT_TREE_ROOT_AND_PURPORTED_CONTRACT_TREE_ROOT_MISMATCH);
    EXPECT_EQ(builder.get_first_failure().message,
              "computed_contract_tree_root doesn't match purported_contract_tree_root");
}

TEST_F(native_private_kernel_inner_tests, private_function_incorrect_function_leaf_index_fails)
{
    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    // Set the leaf index of the function leaf to 10 (the correct value is 1).
    NT::fr const wrong_idx = 10;
    private_inputs.private_call.function_leaf_membership_witness.leaf_index = wrong_idx;

    // Invoke the native private kernel circuit
    DummyBuilder builder = DummyBuilder("private_kernel_tests__private_function_incorrect_contract_leaf_index_fails");
    native_private_kernel_circuit_inner(builder, private_inputs);

    // Assertion checks
    EXPECT_TRUE(builder.failed());
    EXPECT_EQ(builder.get_first_failure().code,
              CircuitErrorCode::PRIVATE_KERNEL__COMPUTED_CONTRACT_TREE_ROOT_AND_PURPORTED_CONTRACT_TREE_ROOT_MISMATCH);
    EXPECT_EQ(builder.get_first_failure().message,
              "computed_contract_tree_root doesn't match purported_contract_tree_root");
}

TEST_F(native_private_kernel_inner_tests, private_function_incorrect_function_leaf_sibling_path_fails)
{
    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    // Change the function leaf's membership proof.
    private_inputs.private_call.function_leaf_membership_witness.sibling_path[0] = fr::random_element();

    // Invoke the native private kernel circuit
    DummyBuilder builder =
        DummyBuilder("private_kernel_tests__private_function_incorrect_contract_leaf_sibling_path_fails");
    native_private_kernel_circuit_inner(builder, private_inputs);

    // Assertion checks
    EXPECT_TRUE(builder.failed());
    EXPECT_EQ(builder.get_first_failure().code,
              CircuitErrorCode::PRIVATE_KERNEL__COMPUTED_CONTRACT_TREE_ROOT_AND_PURPORTED_CONTRACT_TREE_ROOT_MISMATCH);
    EXPECT_EQ(builder.get_first_failure().message,
              "computed_contract_tree_root doesn't match purported_contract_tree_root");
}

// TODO(suyash): Disabled until https://github.com/AztecProtocol/aztec-packages/issues/499 is resolved.
TEST_F(native_private_kernel_inner_tests, DISABLED_private_function_incorrect_call_stack_item_hash_fails)
{
    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    // Set the first call stack member corresponding to the `deposit` function to random scalar.
    private_inputs.private_call.call_stack_item.public_inputs.private_call_stack[0] = NT::fr::random_element();

    // Invoke the native private kernel circuit
    DummyBuilder builder = DummyBuilder("private_kernel_tests__private_function_incorrect_call_stack_item_hash_fails");
    native_private_kernel_circuit_inner(builder, private_inputs);

    // Assertion checks
    EXPECT_TRUE(builder.failed());
    EXPECT_EQ(builder.get_first_failure().code,
              CircuitErrorCode::PRIVATE_KERNEL__CALCULATED_PRIVATE_CALL_HASH_AND_PROVIDED_PRIVATE_CALL_HASH_MISMATCH);
    EXPECT_EQ(builder.get_first_failure().message,
              "calculated private_call_hash does not match provided private_call_hash at the top of the call stack");
}

TEST_F(native_private_kernel_inner_tests, private_kernel_should_fail_if_aggregating_too_many_commitments)
{
    // Negative test to check if push_array_to_array fails if two many commitments are merged together
    DummyBuilder builder = DummyBuilder("should_fail_if_aggregating_too_many_commitments");

    PrivateKernelInputsInner<NT> private_inputs =
        do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    // Mock the previous new commitments to be full, therefore no need commitments can be added
    std::array<fr, MAX_NEW_COMMITMENTS_PER_TX> full_new_commitments{};
    for (size_t i = 0; i < MAX_NEW_COMMITMENTS_PER_TX; ++i) {
        full_new_commitments[i] = i + 1;
    }
    private_inputs.previous_kernel.public_inputs.end.new_commitments = full_new_commitments;
    native_private_kernel_circuit_inner(builder, private_inputs);

    ASSERT_TRUE(builder.failed());
    ASSERT_EQ(builder.get_first_failure().code, CircuitErrorCode::ARRAY_OVERFLOW);
}

/**
 * @brief Test this dummy cbind
 */
TEST_F(native_private_kernel_inner_tests, cbind_private_kernel__dummy_previous_kernel)
{
    auto func = [] { return aztec3::circuits::kernel::private_kernel::utils::dummy_previous_kernel(); };
    auto [actual, expected] = call_func_and_wrapper(func, private_kernel__dummy_previous_kernel);
    // TODO(AD): investigate why direct operator== didn't work
    std::stringstream actual_ss;
    std::stringstream expected_ss;
    actual_ss << actual;
    expected_ss << expected;
    EXPECT_EQ(actual_ss.str(), expected_ss.str());
}

TEST_F(native_private_kernel_inner_tests, native_read_request_bad_request)
{
    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    auto const& contract_address =
        private_inputs.private_call.call_stack_item.public_inputs.call_context.storage_contract_address;

    auto const first_nullifier =
        silo_nullifier<NT>(contract_address, private_inputs.previous_kernel.public_inputs.end.new_nullifiers[0]);
    auto [read_requests,
          read_request_membership_witnesses,
          _transient_read_requests,
          _transient_read_request_membership_witnesses,
          root] = get_random_reads(first_nullifier, contract_address, 2);
    private_inputs.previous_kernel.public_inputs.constants.block_data.private_data_tree_root = root;
    private_inputs.private_call.call_stack_item.public_inputs.historic_block_data.private_data_tree_root = root;

    // tweak read_request so it gives wrong root when paired with its sibling path
    read_requests[1] += 1;

    private_inputs.private_call.call_stack_item.public_inputs.read_requests = read_requests;
    private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;

    // We need to update the previous_kernel's private_call_stack because the current_call_stack_item has changed
    // i.e. we changed the public_inputs->read_requests and public_inputs->historic_private_data_tree_root of the
    // current_call_stack_item
    private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] =
        private_inputs.private_call.call_stack_item.hash();

    DummyBuilder builder = DummyBuilder("native_private_kernel_inner_tests__native_read_request_bad_request");
    auto const& public_inputs = native_private_kernel_circuit_inner(builder, private_inputs);

    validate_no_new_deployed_contract(public_inputs);

    ASSERT_TRUE(builder.failed());
    ASSERT_EQ(builder.get_first_failure().code,
              CircuitErrorCode::PRIVATE_KERNEL__READ_REQUEST_PRIVATE_DATA_ROOT_MISMATCH);
}

TEST_F(native_private_kernel_inner_tests, native_read_request_bad_leaf_index)
{
    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    auto const& contract_address =
        private_inputs.private_call.call_stack_item.public_inputs.call_context.storage_contract_address;

    auto const first_nullifier =
        silo_nullifier<NT>(contract_address, private_inputs.previous_kernel.public_inputs.end.new_nullifiers[0]);
    auto [read_requests,
          read_request_membership_witnesses,
          _transient_read_requests,
          _transient_read_request_membership_witnesses,
          root] = get_random_reads(first_nullifier, contract_address, 2);
    private_inputs.previous_kernel.public_inputs.constants.block_data.private_data_tree_root = root;
    private_inputs.private_call.call_stack_item.public_inputs.historic_block_data.private_data_tree_root = root;

    // tweak leaf index so it gives wrong root when paired with its request and sibling path
    read_request_membership_witnesses[1].leaf_index += 1;
    private_inputs.private_call.call_stack_item.public_inputs.read_requests = read_requests;
    private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;

    // We need to update the previous_kernel's private_call_stack because the current_call_stack_item has changed
    // i.e. we changed the public_inputs->read_requests and public_inputs->historic_private_data_tree_root of the
    // current_call_stack_item
    private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] =
        private_inputs.private_call.call_stack_item.hash();

    DummyBuilder builder = DummyBuilder("native_private_kernel_inner_tests__native_read_request_bad_leaf_index");
    auto const& public_inputs = native_private_kernel_circuit_inner(builder, private_inputs);

    validate_no_new_deployed_contract(public_inputs);

    ASSERT_TRUE(builder.failed());
    ASSERT_EQ(builder.get_first_failure().code,
              CircuitErrorCode::PRIVATE_KERNEL__READ_REQUEST_PRIVATE_DATA_ROOT_MISMATCH);
}

TEST_F(native_private_kernel_inner_tests, native_read_request_bad_sibling_path)
{
    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    auto const& contract_address =
        private_inputs.private_call.call_stack_item.public_inputs.call_context.storage_contract_address;

    auto const first_nullifier =
        silo_nullifier<NT>(contract_address, private_inputs.previous_kernel.public_inputs.end.new_nullifiers[0]);
    auto [read_requests,
          read_request_membership_witnesses,
          _transient_read_requests,
          _transient_read_request_membership_witnesses,
          root] = get_random_reads(first_nullifier, contract_address, 2);
    private_inputs.previous_kernel.public_inputs.constants.block_data.private_data_tree_root = root;
    private_inputs.private_call.call_stack_item.public_inputs.historic_block_data.private_data_tree_root = root;

    // tweak sibling path so it gives wrong root when paired with its request
    read_request_membership_witnesses[1].sibling_path[1] += 1;
    private_inputs.private_call.call_stack_item.public_inputs.read_requests = read_requests;
    private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;

    // We need to update the previous_kernel's private_call_stack because the current_call_stack_item has changed
    // i.e. we changed the public_inputs->read_requests and public_inputs->historic_private_data_tree_root of the
    // current_call_stack_item
    private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] =
        private_inputs.private_call.call_stack_item.hash();

    DummyBuilder builder = DummyBuilder("native_private_kernel_inner_tests__native_read_request_bad_sibling_path");
    auto const& public_inputs = native_private_kernel_circuit_inner(builder, private_inputs);

    validate_no_new_deployed_contract(public_inputs);

    ASSERT_TRUE(builder.failed());
    ASSERT_EQ(builder.get_first_failure().code,
              CircuitErrorCode::PRIVATE_KERNEL__READ_REQUEST_PRIVATE_DATA_ROOT_MISMATCH);
}

TEST_F(native_private_kernel_inner_tests, native_read_request_root_mismatch)
{
    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    auto const& contract_address =
        private_inputs.private_call.call_stack_item.public_inputs.call_context.storage_contract_address;

    // generate two random sets of read requests and mix them so their roots don't match
    auto const first_nullifier =
        silo_nullifier<NT>(contract_address, private_inputs.previous_kernel.public_inputs.end.new_nullifiers[0]);
    auto [read_requests0,
          read_request_membership_witnesses0,
          _transient_read_requests0,
          _transient_read_request_membership_witnesses0,
          root] = get_random_reads(first_nullifier, contract_address, 2);
    private_inputs.previous_kernel.public_inputs.constants.block_data.private_data_tree_root = root;
    private_inputs.private_call.call_stack_item.public_inputs.historic_block_data.private_data_tree_root = root;
    auto [read_requests1,
          read_request_membership_witnesses1,
          _transient_read_requests1,
          _transient_read_request_membership_witnesses1,
          _root] = get_random_reads(first_nullifier, contract_address, 2);
    std::array<NT::fr, MAX_READ_REQUESTS_PER_CALL> bad_requests{};
    std::array<ReadRequestMembershipWitness<NT, PRIVATE_DATA_TREE_HEIGHT>, MAX_READ_REQUESTS_PER_CALL> bad_witnesses;
    // note we are using read_requests0 for some and read_requests1 for others
    bad_requests[0] = read_requests0[0];
    bad_requests[1] = read_requests0[1];
    bad_requests[2] = read_requests1[0];
    bad_requests[3] = read_requests1[1];
    bad_witnesses[0] = read_request_membership_witnesses0[0];
    bad_witnesses[1] = read_request_membership_witnesses0[1];
    bad_witnesses[2] = read_request_membership_witnesses1[0];
    bad_witnesses[3] = read_request_membership_witnesses1[1];
    private_inputs.private_call.call_stack_item.public_inputs.read_requests = bad_requests;
    private_inputs.private_call.read_request_membership_witnesses = bad_witnesses;

    // We need to update the previous_kernel's private_call_stack because the current_call_stack_item has changed
    // i.e. we changed the public_inputs->read_requests and public_inputs->historic_private_data_tree_root of the
    // current_call_stack_item
    private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] =
        private_inputs.private_call.call_stack_item.hash();

    DummyBuilder builder = DummyBuilder("native_private_kernel_inner_tests__native_read_request_root_mismatch");
    auto const& public_inputs = native_private_kernel_circuit_inner(builder, private_inputs);

    validate_no_new_deployed_contract(public_inputs);

    ASSERT_TRUE(builder.failed());
    ASSERT_EQ(builder.get_first_failure().code,
              CircuitErrorCode::PRIVATE_KERNEL__READ_REQUEST_PRIVATE_DATA_ROOT_MISMATCH);
}

TEST_F(native_private_kernel_inner_tests, native_no_read_requests_works)
{
    // no read requests should work

    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    // empty requests
    std::array<fr, MAX_READ_REQUESTS_PER_CALL> const read_requests{};
    std::array<ReadRequestMembershipWitness<NT, PRIVATE_DATA_TREE_HEIGHT>, MAX_READ_REQUESTS_PER_CALL> const
        read_request_membership_witnesses{};
    private_inputs.private_call.call_stack_item.public_inputs.read_requests = read_requests;
    private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;

    // We need to update the previous_kernel's private_call_stack because the current_call_stack_item has changed
    // i.e. we changed the public_inputs->read_requests of the current_call_stack_item
    private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] =
        private_inputs.private_call.call_stack_item.hash();

    DummyBuilder builder = DummyBuilder("native_private_kernel_inner_tests__native_no_read_requests_works");
    auto const& public_inputs = native_private_kernel_circuit_inner(builder, private_inputs);

    validate_no_new_deployed_contract(public_inputs);

    auto failure = builder.get_first_failure();
    if (failure.code != CircuitErrorCode::NO_ERROR) {
        info("failure: ", failure);
    }
    ASSERT_FALSE(builder.failed());

    // non-transient read requests are NOT forwarded
    ASSERT_EQ(array_length(public_inputs.end.read_requests), 0);
}

TEST_F(native_private_kernel_inner_tests, native_one_read_requests_works)
{
    // one read request should work

    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    auto const& contract_address =
        private_inputs.private_call.call_stack_item.public_inputs.call_context.storage_contract_address;

    auto const first_nullifier =
        silo_nullifier<NT>(contract_address, private_inputs.previous_kernel.public_inputs.end.new_nullifiers[0]);
    auto [read_requests,
          read_request_membership_witnesses,
          _transient_read_requests,
          _transient_read_request_membership_witnesses,
          root] = get_random_reads(first_nullifier, contract_address, 1);
    private_inputs.previous_kernel.public_inputs.constants.block_data.private_data_tree_root = root;
    private_inputs.private_call.call_stack_item.public_inputs.historic_block_data.private_data_tree_root = root;
    private_inputs.private_call.call_stack_item.public_inputs.read_requests = read_requests;
    private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;

    // We need to update the previous_kernel's private_call_stack because the current_call_stack_item has changed
    // i.e. we changed the public_inputs->read_requests of the current_call_stack_item
    private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] =
        private_inputs.private_call.call_stack_item.hash();

    DummyBuilder builder = DummyBuilder("native_private_kernel_inner_tests__native_one_read_requests_works");
    auto const& public_inputs = native_private_kernel_circuit_inner(builder, private_inputs);

    validate_no_new_deployed_contract(public_inputs);

    auto failure = builder.get_first_failure();
    if (failure.code != CircuitErrorCode::NO_ERROR) {
        info("failure: ", failure);
    }
    ASSERT_FALSE(builder.failed());

    // non-transient read requests are NOT forwarded
    ASSERT_EQ(array_length(public_inputs.end.read_requests), 0);
}

TEST_F(native_private_kernel_inner_tests, native_two_read_requests_works)
{
    // two read requests should work

    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    auto const& contract_address =
        private_inputs.private_call.call_stack_item.public_inputs.call_context.storage_contract_address;

    auto const first_nullifier =
        silo_nullifier<NT>(contract_address, private_inputs.previous_kernel.public_inputs.end.new_nullifiers[0]);
    auto [read_requests,
          read_request_membership_witnesses,
          _transient_read_requests,
          _transient_read_request_membership_witnesses,
          root] = get_random_reads(first_nullifier, contract_address, 2);
    private_inputs.previous_kernel.public_inputs.constants.block_data.private_data_tree_root = root;
    private_inputs.private_call.call_stack_item.public_inputs.historic_block_data.private_data_tree_root = root;
    private_inputs.private_call.call_stack_item.public_inputs.read_requests = read_requests;
    private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;

    // We need to update the previous_kernel's private_call_stack because the current_call_stack_item has changed
    // i.e. we changed the public_inputs->read_requests of the current_call_stack_item
    private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] =
        private_inputs.private_call.call_stack_item.hash();

    DummyBuilder builder = DummyBuilder("native_private_kernel_inner_tests__native_two_read_requests_works");
    auto const& public_inputs = native_private_kernel_circuit_inner(builder, private_inputs);

    validate_no_new_deployed_contract(public_inputs);

    auto failure = builder.get_first_failure();
    if (failure.code != CircuitErrorCode::NO_ERROR) {
        info("failure: ", failure);
    }
    ASSERT_FALSE(builder.failed());

    // non-transient read requests are NOT forwarded
    ASSERT_EQ(array_length(public_inputs.end.read_requests), 0);
}

TEST_F(native_private_kernel_inner_tests, native_max_read_requests_works)
{
    // max read requests should work

    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    auto const& contract_address =
        private_inputs.private_call.call_stack_item.public_inputs.call_context.storage_contract_address;

    auto const first_nullifier =
        silo_nullifier<NT>(contract_address, private_inputs.previous_kernel.public_inputs.end.new_nullifiers[0]);
    auto [read_requests,
          read_request_membership_witnesses,
          _transient_read_requests,
          _transient_read_request_membership_witnesses,
          root] = get_random_reads(first_nullifier, contract_address, MAX_READ_REQUESTS_PER_CALL);
    private_inputs.previous_kernel.public_inputs.constants.block_data.private_data_tree_root = root;
    private_inputs.private_call.call_stack_item.public_inputs.historic_block_data.private_data_tree_root = root;
    private_inputs.private_call.call_stack_item.public_inputs.read_requests = read_requests;
    private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;

    // We need to update the previous_kernel's private_call_stack because the current_call_stack_item has changed
    // i.e. we changed the public_inputs->read_requests and public_inputs->historic_private_data_tree_root of the
    // current_call_stack_item
    private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] =
        private_inputs.private_call.call_stack_item.hash();

    DummyBuilder builder = DummyBuilder("native_private_kernel_inner_tests__native_max_read_requests_works");
    auto const& public_inputs = native_private_kernel_circuit_inner(builder, private_inputs);

    validate_no_new_deployed_contract(public_inputs);

    auto failure = builder.get_first_failure();
    if (failure.code != CircuitErrorCode::NO_ERROR) {
        info("failure: ", failure);
    }
    ASSERT_FALSE(builder.failed());

    // non-transient read requests are NOT forwarded
    ASSERT_EQ(array_length(public_inputs.end.read_requests), 0);
}

TEST_F(native_private_kernel_inner_tests, native_one_transient_read_requests_works)
{
    // one transient read request should work

    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    auto const& contract_address =
        private_inputs.private_call.call_stack_item.public_inputs.call_context.storage_contract_address;

    auto const first_nullifier =
        silo_nullifier<NT>(contract_address, private_inputs.previous_kernel.public_inputs.end.new_nullifiers[0]);
    auto [read_requests,
          read_request_membership_witnesses,
          transient_read_requests,
          transient_read_request_membership_witnesses,
          root] = get_random_reads(first_nullifier, contract_address, 1);
    private_inputs.private_call.call_stack_item.public_inputs.historic_block_data.private_data_tree_root = root;

    // Make the read request transient
    read_requests[0] = transient_read_requests[0];
    read_request_membership_witnesses[0] = transient_read_request_membership_witnesses[0];
    private_inputs.private_call.call_stack_item.public_inputs.read_requests = read_requests;
    private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;

    // We need to update the previous_kernel's private_call_stack because the current_call_stack_item has changed
    // i.e. we changed the public_inputs->read_requests and public_inputs->historic_private_data_tree_root of the
    // current_call_stack_item
    private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] =
        private_inputs.private_call.call_stack_item.hash();

    DummyBuilder builder = DummyBuilder("native_private_kernel_inner_tests__native_one_transient_read_requests_works");
    auto const& public_inputs = native_private_kernel_circuit_inner(builder, private_inputs);

    validate_no_new_deployed_contract(public_inputs);

    auto failure = builder.get_first_failure();
    if (failure.code != CircuitErrorCode::NO_ERROR) {
        info("failure: ", failure);
    }
    ASSERT_FALSE(builder.failed());

    ASSERT_EQ(array_length(public_inputs.end.read_requests), 1);  // transient read request gets forwarded
}

TEST_F(native_private_kernel_inner_tests, native_max_read_requests_one_transient_works)
{
    // max read requests with one transient should work

    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    auto const& contract_address =
        private_inputs.private_call.call_stack_item.public_inputs.call_context.storage_contract_address;

    auto const first_nullifier =
        silo_nullifier<NT>(contract_address, private_inputs.previous_kernel.public_inputs.end.new_nullifiers[0]);
    auto [read_requests,
          read_request_membership_witnesses,
          transient_read_requests,
          transient_read_request_membership_witnesses,
          root] = get_random_reads(first_nullifier, contract_address, MAX_READ_REQUESTS_PER_CALL);
    private_inputs.previous_kernel.public_inputs.constants.block_data.private_data_tree_root = root;
    private_inputs.private_call.call_stack_item.public_inputs.historic_block_data.private_data_tree_root = root;

    // Make the read request at position 1 transient
    read_requests[1] = transient_read_requests[1];
    read_request_membership_witnesses[1] = transient_read_request_membership_witnesses[1];
    private_inputs.private_call.call_stack_item.public_inputs.read_requests = read_requests;
    private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;

    // We need to update the previous_kernel's private_call_stack because the current_call_stack_item has changed
    // i.e. we changed the public_inputs->read_requests and public_inputs->historic_private_data_tree_root of the
    // current_call_stack_item
    private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] =
        private_inputs.private_call.call_stack_item.hash();

    DummyBuilder builder =
        DummyBuilder("native_private_kernel_inner_tests__native_max_read_requests_one_transient_works");
    auto const& public_inputs = native_private_kernel_circuit_inner(builder, private_inputs);

    validate_no_new_deployed_contract(public_inputs);

    auto failure = builder.get_first_failure();
    if (failure.code != CircuitErrorCode::NO_ERROR) {
        info("failure: ", failure);
    }
    ASSERT_FALSE(builder.failed());

    // transient read request gets forwarded
    ASSERT_EQ(array_length(public_inputs.end.read_requests), 1);
}

TEST_F(native_private_kernel_inner_tests, native_max_read_requests_all_transient_works)
{
    // max read requests with all transient should work

    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    auto const& contract_address =
        private_inputs.private_call.call_stack_item.public_inputs.call_context.storage_contract_address;

    auto const first_nullifier =
        silo_nullifier<NT>(contract_address, private_inputs.previous_kernel.public_inputs.end.new_nullifiers[0]);
    auto [read_requests,
          read_request_membership_witnesses,
          transient_read_requests,
          transient_read_request_membership_witnesses,
          root] = get_random_reads(first_nullifier, contract_address, MAX_READ_REQUESTS_PER_CALL);
    private_inputs.previous_kernel.public_inputs.constants.block_data.private_data_tree_root = root;
    private_inputs.private_call.call_stack_item.public_inputs.historic_block_data.private_data_tree_root = root;
    private_inputs.private_call.call_stack_item.public_inputs.read_requests = transient_read_requests;
    private_inputs.private_call.read_request_membership_witnesses = transient_read_request_membership_witnesses;

    // We need to update the previous_kernel's private_call_stack because the current_call_stack_item has changed
    // i.e. we changed the public_inputs->read_requests and public_inputs->historic_private_data_tree_root of the
    // current_call_stack_item
    private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] =
        private_inputs.private_call.call_stack_item.hash();

    DummyBuilder builder =
        DummyBuilder("native_private_kernel_inner_tests__native_max_read_requests_one_transient_works");
    auto const& public_inputs = native_private_kernel_circuit_inner(builder, private_inputs);

    validate_no_new_deployed_contract(public_inputs);

    auto failure = builder.get_first_failure();
    if (failure.code != CircuitErrorCode::NO_ERROR) {
        info("failure: ", failure);
    }
    ASSERT_FALSE(builder.failed());

    // transient read request all get forwarded
    ASSERT_EQ(array_length(public_inputs.end.read_requests), MAX_READ_REQUESTS_PER_CALL);
}

TEST_F(native_private_kernel_inner_tests, native_logs_are_hashed_as_expected)
{
    std::array<NT::fr, NUM_FIELDS_PER_SHA256> const& encrypted_logs_hash = { NT::fr(16), NT::fr(69) };
    NT::fr const& encrypted_log_preimages_length = NT::fr(100);
    std::array<NT::fr, NUM_FIELDS_PER_SHA256> const& unencrypted_logs_hash = { NT::fr(26), NT::fr(47) };
    NT::fr const& unencrypted_log_preimages_length = NT::fr(50);
    std::array<NT::fr, NUM_FIELDS_PER_SHA256> const& public_inputs_encrypted_logs_hash = { NT::fr(80), NT::fr(429) };
    NT::fr const& public_inputs_encrypted_log_preimages_length = NT::fr(13);
    std::array<NT::fr, NUM_FIELDS_PER_SHA256> const& public_inputs_unencrypted_logs_hash = { NT::fr(956), NT::fr(112) };
    NT::fr const& public_inputs_unencrypted_log_preimages_length = NT::fr(24);

    auto private_inputs = do_private_call_get_kernel_inputs_inner(false,
                                                                  deposit,
                                                                  standard_test_args(),
                                                                  encrypted_logs_hash,
                                                                  unencrypted_logs_hash,
                                                                  encrypted_log_preimages_length,
                                                                  unencrypted_log_preimages_length,
                                                                  public_inputs_encrypted_logs_hash,
                                                                  public_inputs_unencrypted_logs_hash,
                                                                  public_inputs_encrypted_log_preimages_length,
                                                                  public_inputs_unencrypted_log_preimages_length);

    DummyBuilder builder = DummyBuilder("native_private_kernel_inner_tests__native_logs_are_hashed_as_expected");
    auto const& public_inputs = native_private_kernel_circuit_inner(builder, private_inputs);

    ASSERT_EQ(public_inputs.end.encrypted_log_preimages_length,
              encrypted_log_preimages_length + public_inputs_encrypted_log_preimages_length);
    ASSERT_EQ(public_inputs.end.unencrypted_log_preimages_length,
              unencrypted_log_preimages_length + public_inputs_unencrypted_log_preimages_length);

    auto const& expected_encrypted_logs_hash = accumulate_sha256<NT>({ public_inputs_encrypted_logs_hash[0],
                                                                       public_inputs_encrypted_logs_hash[1],
                                                                       encrypted_logs_hash[0],
                                                                       encrypted_logs_hash[1] });

    ASSERT_EQ(public_inputs.end.encrypted_logs_hash, expected_encrypted_logs_hash);

    auto const& expected_unencrypted_logs_hash = accumulate_sha256<NT>({ public_inputs_unencrypted_logs_hash[0],
                                                                         public_inputs_unencrypted_logs_hash[1],
                                                                         unencrypted_logs_hash[0],
                                                                         unencrypted_logs_hash[1] });

    ASSERT_EQ(public_inputs.end.unencrypted_logs_hash, expected_unencrypted_logs_hash);
}

}  // namespace aztec3::circuits::kernel::private_kernel
