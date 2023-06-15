#include "c_bind.h"
#include "testing_harness.hpp"

#include "aztec3/circuits/apps/test_apps/basic_contract_deployment/basic_contract_deployment.hpp"
#include "aztec3/circuits/apps/test_apps/escrow/deposit.hpp"
#include "aztec3/utils/circuit_errors.hpp"

#include <barretenberg/barretenberg.hpp>

#include <gtest/gtest.h>

#include <cstdint>

namespace {

using aztec3::circuits::apps::test_apps::basic_contract_deployment::constructor;
using aztec3::circuits::apps::test_apps::escrow::deposit;

using aztec3::circuits::kernel::private_kernel::testing_harness::do_private_call_get_kernel_inputs_init;
using aztec3::circuits::kernel::private_kernel::testing_harness::do_private_call_get_kernel_inputs_inner;
using aztec3::circuits::kernel::private_kernel::testing_harness::get_random_reads;
using aztec3::circuits::kernel::private_kernel::testing_harness::validate_deployed_contract_address;
using aztec3::circuits::kernel::private_kernel::testing_harness::validate_no_new_deployed_contract;
using aztec3::utils::CircuitErrorCode;

}  // namespace

namespace aztec3::circuits::kernel::private_kernel {

class native_private_kernel_init_tests : public ::testing::Test {
  protected:
    static void SetUpTestSuite() { barretenberg::srs::init_crs_factory("../barretenberg/cpp/srs_db/ignition"); }
};

class native_private_kernel_inner_tests : public ::testing::Test {
  protected:
    static void SetUpTestSuite() { barretenberg::srs::init_crs_factory("../barretenberg/cpp/srs_db/ignition"); }
};

/**
 **************************************************************
 * Native initial private kernel circuit tests.
 **************************************************************
 */

/**
 * @brief Some private circuit simulation (`deposit`, in this case)
 */
TEST_F(native_private_kernel_init_tests, deposit)
{
    NT::fr const& amount = 5;
    NT::fr const& asset_id = 1;
    NT::fr const& memo = 999;
    std::array<NT::fr, 2> const& encrypted_logs_hash = { NT::fr(16), NT::fr(69) };
    NT::fr const& encrypted_log_preimages_length = NT::fr(100);

    auto const& private_inputs = do_private_call_get_kernel_inputs_init(
        false, deposit, { amount, asset_id, memo }, encrypted_logs_hash, encrypted_log_preimages_length);
    DummyComposer composer = DummyComposer("private_kernel_tests__native_deposit");
    auto const& public_inputs = native_private_kernel_circuit_initial(composer, private_inputs);

    EXPECT_TRUE(validate_no_new_deployed_contract(public_inputs));

    // Check the first nullifier is hash of the signed tx request
    ASSERT_EQ(public_inputs.end.new_nullifiers[0], private_inputs.tx_request.hash());

    // Log preimages length should increase by `encrypted_log_preimages_length` from private input
    ASSERT_EQ(public_inputs.end.encrypted_log_preimages_length, encrypted_log_preimages_length);
    // Since there were no unencrypted logs, their length should be 0
    ASSERT_EQ(public_inputs.end.unencrypted_log_preimages_length, fr(0));

    // Encrypted logs hash should be a sha256 hash of a 0 value and the `encrypted_logs_hash` from private input
    auto const& expected_encrypted_logs_hash =
        accumulate_sha256<NT>({ fr(0), fr(0), encrypted_logs_hash[0], encrypted_logs_hash[1] });
    ASSERT_EQ(public_inputs.end.encrypted_logs_hash, expected_encrypted_logs_hash);

    // Unencrypted logs hash should be a sha256 hash of 2 zero values
    auto const& expected_unencrypted_logs_hash = accumulate_sha256<NT>({ fr(0), fr(0), fr(0), fr(0) });
    ASSERT_EQ(public_inputs.end.unencrypted_logs_hash, expected_unencrypted_logs_hash);

    // Assert that composer doesn't give any errors
    ASSERT_FALSE(composer.failed());
    EXPECT_EQ(composer.get_first_failure().message, "");
    EXPECT_EQ(composer.get_first_failure().code, CircuitErrorCode::NO_ERROR);
}

/**
 * @brief Some private circuit simulation (`constructor`, in this case)
 */
TEST_F(native_private_kernel_init_tests, basic_contract_deployment)
{
    NT::fr const arg0 = 5;
    NT::fr const arg1 = 1;
    NT::fr const arg2 = 999;

    auto const& private_inputs = do_private_call_get_kernel_inputs_init(true, constructor, { arg0, arg1, arg2 });
    DummyComposer composer = DummyComposer("private_kernel_tests__native_basic_contract_deployment");
    auto const& public_inputs = native_private_kernel_circuit_initial(composer, private_inputs);

    EXPECT_TRUE(validate_deployed_contract_address(private_inputs, public_inputs));

    // Since there are no logs, log preimages length should be 0 and both logs hashes should be a sha256 hash of 2 zero
    // values
    ASSERT_EQ(public_inputs.end.encrypted_log_preimages_length, fr(0));
    ASSERT_EQ(public_inputs.end.unencrypted_log_preimages_length, fr(0));

    auto const& expected_logs_hash = accumulate_sha256<NT>({ fr(0), fr(0), fr(0), fr(0) });

    ASSERT_EQ(public_inputs.end.encrypted_logs_hash, expected_logs_hash);
    ASSERT_EQ(public_inputs.end.encrypted_logs_hash, expected_logs_hash);

    // Assert that composer doesn't give any errors
    ASSERT_FALSE(composer.failed());
    EXPECT_EQ(composer.get_first_failure().message, "");
    EXPECT_EQ(composer.get_first_failure().code, CircuitErrorCode::NO_ERROR);
}

// TODO(suyash): Disabled until https://github.com/AztecProtocol/aztec-packages/issues/499 is resolved.
TEST_F(native_private_kernel_init_tests, DISABLED_contract_deployment_call_stack_item_hash_mismatch_fails)
{
    NT::fr const arg0 = 5;
    NT::fr const arg1 = 1;
    NT::fr const arg2 = 999;

    auto private_inputs = do_private_call_get_kernel_inputs_init(true, constructor, { arg0, arg1, arg2 });

    // Randomise the second item in the private call stack (i.e. hash of the private call item).
    private_inputs.private_call.call_stack_item.public_inputs.private_call_stack[1] = NT::fr::random_element();

    DummyComposer composer =
        DummyComposer("private_kernel_tests__contract_deployment_call_stack_item_hash_mismatch_fails");
    native_private_kernel_circuit_initial(composer, private_inputs);

    EXPECT_EQ(composer.failed(), true);
    EXPECT_EQ(composer.get_first_failure().code,
              CircuitErrorCode::PRIVATE_KERNEL__PRIVATE_CALL_STACK_ITEM_HASH_MISMATCH);
}

TEST_F(native_private_kernel_init_tests, contract_deployment_incorrect_constructor_vk_hash_fails)
{
    NT::fr const arg0 = 5;
    NT::fr const arg1 = 1;
    NT::fr const arg2 = 999;

    auto private_inputs = do_private_call_get_kernel_inputs_init(true, constructor, { arg0, arg1, arg2 });

    // Pollute the constructor vk hash in the tx_request.
    private_inputs.tx_request.tx_context.contract_deployment_data.constructor_vk_hash = NT::fr::random_element();

    DummyComposer composer =
        DummyComposer("private_kernel_tests__contract_deployment_incorrect_constructor_vk_hash_fails");
    native_private_kernel_circuit_initial(composer, private_inputs);

    EXPECT_EQ(composer.failed(), true);
    EXPECT_EQ(composer.get_first_failure().code, CircuitErrorCode::PRIVATE_KERNEL__INVALID_CONSTRUCTOR_VK_HASH);
    EXPECT_EQ(composer.get_first_failure().message, "constructor_vk_hash doesn't match private_call_vk_hash");
}

TEST_F(native_private_kernel_init_tests, contract_deployment_incorrect_contract_address_fails)
{
    NT::fr const arg0 = 5;
    NT::fr const arg1 = 1;
    NT::fr const arg2 = 999;

    auto private_inputs = do_private_call_get_kernel_inputs_init(true, constructor, { arg0, arg1, arg2 });

    // Modify the contract address in appropriate places.
    const fr random_address = NT::fr::random_element();
    private_inputs.private_call.call_stack_item.public_inputs.call_context.storage_contract_address = random_address;
    private_inputs.tx_request.to = random_address;
    private_inputs.private_call.call_stack_item.contract_address = random_address;

    DummyComposer composer =
        DummyComposer("private_kernel_tests__contract_deployment_incorrect_contract_address_fails");
    native_private_kernel_circuit_initial(composer, private_inputs);

    EXPECT_EQ(composer.failed(), true);
    EXPECT_EQ(composer.get_first_failure().code, CircuitErrorCode::PRIVATE_KERNEL__INVALID_CONTRACT_ADDRESS);
    EXPECT_EQ(composer.get_first_failure().message, "contract address supplied doesn't match derived address");
}

TEST_F(native_private_kernel_init_tests, contract_deployment_contract_address_mismatch_fails)
{
    NT::fr const arg0 = 5;
    NT::fr const arg1 = 1;
    NT::fr const arg2 = 999;

    auto private_inputs = do_private_call_get_kernel_inputs_init(true, constructor, { arg0, arg1, arg2 });

    // Modify the storage_contract_address.
    const auto random_contract_address = NT::fr::random_element();
    private_inputs.private_call.call_stack_item.public_inputs.call_context.storage_contract_address =
        random_contract_address;
    private_inputs.private_call.call_stack_item.contract_address = random_contract_address;

    // Invoke the native private kernel circuit
    DummyComposer composer = DummyComposer("private_kernel_tests__contract_deployment_contract_address_mismatch_fails");
    native_private_kernel_circuit_initial(composer, private_inputs);

    // Assertion checks
    EXPECT_TRUE(composer.failed());
    EXPECT_EQ(composer.get_first_failure().code,
              CircuitErrorCode::PRIVATE_KERNEL__USER_INTENT_MISMATCH_BETWEEN_TX_REQUEST_AND_CALL_STACK_ITEM);
    EXPECT_EQ(composer.get_first_failure().message,
              "user's intent does not match initial private call (tx_request.to must match "
              "call_stack_item.contract_address)");
}

TEST_F(native_private_kernel_init_tests, contract_deployment_function_data_mismatch_fails)
{
    NT::fr const arg0 = 5;
    NT::fr const arg1 = 1;
    NT::fr const arg2 = 999;

    auto private_inputs = do_private_call_get_kernel_inputs_init(true, constructor, { arg0, arg1, arg2 });

    // Modify the function selector in function data.
    private_inputs.tx_request.function_data.function_selector = numeric::random::get_engine().get_random_uint32();

    // Invoke the native private kernel circuit
    DummyComposer composer = DummyComposer("private_kernel_tests__contract_deployment_function_data_mismatch_fails");
    native_private_kernel_circuit_initial(composer, private_inputs);

    // Assertion checks
    EXPECT_TRUE(composer.failed());
    EXPECT_EQ(composer.get_first_failure().code,
              CircuitErrorCode::PRIVATE_KERNEL__USER_INTENT_MISMATCH_BETWEEN_TX_REQUEST_AND_CALL_STACK_ITEM);
    EXPECT_EQ(composer.get_first_failure().message,
              "user's intent does not match initial private call (tx_request.function_data must match "
              "call_stack_item.function_data)");
}

TEST_F(native_private_kernel_init_tests, contract_deployment_args_hash_mismatch_fails)
{
    NT::fr const arg0 = 5;
    NT::fr const arg1 = 1;
    NT::fr const arg2 = 999;

    auto private_inputs = do_private_call_get_kernel_inputs_init(true, constructor, { arg0, arg1, arg2 });

    // Modify the args hash in tx request.
    private_inputs.tx_request.args_hash = NT::fr::random_element();

    // Invoke the native private kernel circuit
    DummyComposer composer = DummyComposer("private_kernel_tests__contract_deployment_args_hash_mismatch_fails");
    native_private_kernel_circuit_initial(composer, private_inputs);

    // Assertion checks
    EXPECT_TRUE(composer.failed());
    EXPECT_EQ(composer.get_first_failure().code,
              CircuitErrorCode::PRIVATE_KERNEL__USER_INTENT_MISMATCH_BETWEEN_TX_REQUEST_AND_CALL_STACK_ITEM);
    EXPECT_EQ(composer.get_first_failure().message,
              "user's intent does not match initial private call (tx_request.args must match "
              "call_stack_item.public_inputs.args)");
}

TEST_F(native_private_kernel_init_tests, private_function_is_private_false_fails)
{
    NT::fr const arg0 = 5;
    NT::fr const arg1 = 1;
    NT::fr const arg2 = 999;

    auto private_inputs = do_private_call_get_kernel_inputs_init(false, deposit, { arg0, arg1, arg2 });

    // Set is_private in function data to false.
    private_inputs.private_call.call_stack_item.function_data.is_private = false;

    // Invoke the native private kernel circuit
    DummyComposer composer = DummyComposer("private_kernel_tests__private_function_is_private_false_fails");
    native_private_kernel_circuit_initial(composer, private_inputs);

    // Assertion checks
    EXPECT_TRUE(composer.failed());
    EXPECT_EQ(composer.get_first_failure().code,
              CircuitErrorCode::PRIVATE_KERNEL__NON_PRIVATE_FUNCTION_EXECUTED_WITH_PRIVATE_KERNEL);
    EXPECT_EQ(composer.get_first_failure().message,
              "Cannot execute a non-private function with the private kernel circuit");
}


TEST_F(native_private_kernel_init_tests, private_function_static_call_fails)
{
    NT::fr const arg0 = 5;
    NT::fr const arg1 = 1;
    NT::fr const arg2 = 999;

    auto private_inputs = do_private_call_get_kernel_inputs_init(false, deposit, { arg0, arg1, arg2 });

    // Set is_static_call to true.
    private_inputs.private_call.call_stack_item.public_inputs.call_context.is_static_call = true;

    // Invoke the native private kernel circuit
    DummyComposer composer = DummyComposer("private_kernel_tests__private_function_static_call_fails");
    native_private_kernel_circuit_initial(composer, private_inputs);

    // Assertion checks
    EXPECT_TRUE(composer.failed());
    EXPECT_EQ(composer.get_first_failure().code, CircuitErrorCode::PRIVATE_KERNEL__UNSUPPORTED_OP);
    EXPECT_EQ(composer.get_first_failure().message, "Users cannot make a static call");
}

TEST_F(native_private_kernel_init_tests, private_function_delegate_call_fails)
{
    NT::fr const arg0 = 5;
    NT::fr const arg1 = 1;
    NT::fr const arg2 = 999;

    auto private_inputs = do_private_call_get_kernel_inputs_init(false, deposit, { arg0, arg1, arg2 });

    // Set is_delegate_call to true.
    private_inputs.private_call.call_stack_item.public_inputs.call_context.is_delegate_call = true;

    // Invoke the native private kernel circuit
    DummyComposer composer = DummyComposer("private_kernel_tests__private_function_delegate_call_fails");
    native_private_kernel_circuit_initial(composer, private_inputs);

    // Assertion checks
    EXPECT_TRUE(composer.failed());
    EXPECT_EQ(composer.get_first_failure().code, CircuitErrorCode::PRIVATE_KERNEL__UNSUPPORTED_OP);
    EXPECT_EQ(composer.get_first_failure().message, "Users cannot make a delegatecall");
}

TEST_F(native_private_kernel_init_tests, private_function_incorrect_storage_contract_address_fails)
{
    NT::fr const arg0 = 5;
    NT::fr const arg1 = 1;
    NT::fr const arg2 = 999;

    auto private_inputs = do_private_call_get_kernel_inputs_init(false, deposit, { arg0, arg1, arg2 });

    // Set the storage_contract_address to a random scalar.
    private_inputs.private_call.call_stack_item.public_inputs.call_context.storage_contract_address =
        NT::fr::random_element();

    // Invoke the native private kernel circuit
    DummyComposer composer =
        DummyComposer("private_kernel_tests__private_function_incorrect_storage_contract_address_fails");
    native_private_kernel_circuit_initial(composer, private_inputs);

    // Assertion checks
    EXPECT_TRUE(composer.failed());
    EXPECT_EQ(composer.get_first_failure().code, CircuitErrorCode::PRIVATE_KERNEL__CONTRACT_ADDRESS_MISMATCH);
    EXPECT_EQ(composer.get_first_failure().message, "Storage contract address must be that of the called contract");
}

TEST_F(native_private_kernel_init_tests, native_read_request_bad_request)
{
    NT::fr const& amount = 5;
    NT::fr const& asset_id = 1;
    NT::fr const& memo = 999;

    auto private_inputs = do_private_call_get_kernel_inputs_init(false, deposit, { amount, asset_id, memo });

    auto const& contract_address =
        private_inputs.private_call.call_stack_item.public_inputs.call_context.storage_contract_address;

    auto [read_requests, read_request_membership_witnesses, root] = get_random_reads(contract_address, 2);
    private_inputs.private_call.call_stack_item.public_inputs.historic_private_data_tree_root = root;

    // tweak read_request so it gives wrong root when paired with its sibling path
    read_requests[1] += 1;

    private_inputs.private_call.call_stack_item.public_inputs.read_requests = read_requests;
    private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;

    DummyComposer composer = DummyComposer("native_private_kernel_init_tests__native_read_request_bad_request");
    auto const& public_inputs = native_private_kernel_circuit_initial(composer, private_inputs);

    validate_no_new_deployed_contract(public_inputs);

    ASSERT(composer.failed());
    ASSERT_EQ(composer.get_first_failure().code,
              CircuitErrorCode::PRIVATE_KERNEL__READ_REQUEST_PRIVATE_DATA_ROOT_MISMATCH);

    // Check the first nullifier is hash of the signed tx request
    ASSERT_EQ(public_inputs.end.new_nullifiers[0], private_inputs.tx_request.hash());
}

TEST_F(native_private_kernel_init_tests, native_read_request_bad_leaf_index)
{
    NT::fr const& amount = 5;
    NT::fr const& asset_id = 1;
    NT::fr const& memo = 999;

    auto private_inputs = do_private_call_get_kernel_inputs_init(false, deposit, { amount, asset_id, memo });

    auto const& contract_address =
        private_inputs.private_call.call_stack_item.public_inputs.call_context.storage_contract_address;

    auto [read_requests, read_request_membership_witnesses, root] = get_random_reads(contract_address, 2);
    private_inputs.private_call.call_stack_item.public_inputs.historic_private_data_tree_root = root;

    // tweak leaf index so it gives wrong root when paired with its request and sibling path
    read_request_membership_witnesses[1].leaf_index += 1;
    private_inputs.private_call.call_stack_item.public_inputs.read_requests = read_requests;
    private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;

    DummyComposer composer = DummyComposer("native_private_kernel_init_tests__native_read_request_bad_leaf_index");
    auto const& public_inputs = native_private_kernel_circuit_initial(composer, private_inputs);

    validate_no_new_deployed_contract(public_inputs);

    ASSERT(composer.failed());
    ASSERT_EQ(composer.get_first_failure().code,
              CircuitErrorCode::PRIVATE_KERNEL__READ_REQUEST_PRIVATE_DATA_ROOT_MISMATCH);

    // Check the first nullifier is hash of the signed tx request
    ASSERT_EQ(public_inputs.end.new_nullifiers[0], private_inputs.tx_request.hash());
}

TEST_F(native_private_kernel_init_tests, native_read_request_bad_sibling_path)
{
    NT::fr const& amount = 5;
    NT::fr const& asset_id = 1;
    NT::fr const& memo = 999;

    auto private_inputs = do_private_call_get_kernel_inputs_init(false, deposit, { amount, asset_id, memo });

    auto const& contract_address =
        private_inputs.private_call.call_stack_item.public_inputs.call_context.storage_contract_address;

    auto [read_requests, read_request_membership_witnesses, root] = get_random_reads(contract_address, 2);
    private_inputs.private_call.call_stack_item.public_inputs.historic_private_data_tree_root = root;

    // tweak sibling path so it gives wrong root when paired with its request
    read_request_membership_witnesses[1].sibling_path[1] += 1;
    private_inputs.private_call.call_stack_item.public_inputs.read_requests = read_requests;
    private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;

    DummyComposer composer = DummyComposer("native_private_kernel_init_tests__native_read_request_bad_sibling_path");
    auto const& public_inputs = native_private_kernel_circuit_initial(composer, private_inputs);

    validate_no_new_deployed_contract(public_inputs);

    ASSERT(composer.failed());
    ASSERT_EQ(composer.get_first_failure().code,
              CircuitErrorCode::PRIVATE_KERNEL__READ_REQUEST_PRIVATE_DATA_ROOT_MISMATCH);

    // Check the first nullifier is hash of the signed tx request
    ASSERT_EQ(public_inputs.end.new_nullifiers[0], private_inputs.tx_request.hash());
}

TEST_F(native_private_kernel_init_tests, native_read_request_root_mismatch)
{
    NT::fr const& amount = 5;
    NT::fr const& asset_id = 1;
    NT::fr const& memo = 999;

    auto private_inputs = do_private_call_get_kernel_inputs_init(false, deposit, { amount, asset_id, memo });

    auto const& contract_address =
        private_inputs.private_call.call_stack_item.public_inputs.call_context.storage_contract_address;

    // generate two random sets of read requests and mix them so their roots don't match
    auto [read_requests0, read_request_membership_witnesses0, root] = get_random_reads(contract_address, 2);
    private_inputs.private_call.call_stack_item.public_inputs.historic_private_data_tree_root = root;
    auto [read_requests1, read_request_membership_witnesses1, _root] = get_random_reads(contract_address, 2);
    std::array<NT::fr, READ_REQUESTS_LENGTH> bad_requests{};
    std::array<MembershipWitness<NT, PRIVATE_DATA_TREE_HEIGHT>, READ_REQUESTS_LENGTH> bad_witnesses;
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

    DummyComposer composer = DummyComposer("native_private_kernel_init_tests__native_read_request_root_mismatch");
    auto const& public_inputs = native_private_kernel_circuit_initial(composer, private_inputs);

    validate_no_new_deployed_contract(public_inputs);

    ASSERT_TRUE(composer.failed());
    ASSERT_EQ(composer.get_first_failure().code,
              CircuitErrorCode::PRIVATE_KERNEL__READ_REQUEST_PRIVATE_DATA_ROOT_MISMATCH);

    // Check the first nullifier is hash of the signed tx request
    ASSERT_EQ(public_inputs.end.new_nullifiers[0], private_inputs.tx_request.hash());
}

TEST_F(native_private_kernel_init_tests, native_no_read_requests_works)
{
    // no read requests should work

    NT::fr const& amount = 5;
    NT::fr const& asset_id = 1;
    NT::fr const& memo = 999;

    auto private_inputs = do_private_call_get_kernel_inputs_init(false, deposit, { amount, asset_id, memo });

    // empty requests
    std::array<fr, READ_REQUESTS_LENGTH> const read_requests = zero_array<fr, READ_REQUESTS_LENGTH>();
    std::array<MembershipWitness<NT, PRIVATE_DATA_TREE_HEIGHT>, READ_REQUESTS_LENGTH> const
        read_request_membership_witnesses{};
    private_inputs.private_call.call_stack_item.public_inputs.read_requests = read_requests;
    private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;

    DummyComposer composer = DummyComposer("native_private_kernel_init_tests__native_no_read_requests_works");
    auto const& public_inputs = native_private_kernel_circuit_initial(composer, private_inputs);

    validate_no_new_deployed_contract(public_inputs);

    auto failure = composer.get_first_failure();
    if (failure.code != CircuitErrorCode::NO_ERROR) {
        info("failure: ", failure);
    }
    ASSERT_FALSE(composer.failed());

    // Check the first nullifier is hash of the signed tx request
    ASSERT_EQ(public_inputs.end.new_nullifiers[0], private_inputs.tx_request.hash());
}

TEST_F(native_private_kernel_init_tests, native_one_read_requests_works)
{
    // one read request should work

    NT::fr const& amount = 5;
    NT::fr const& asset_id = 1;
    NT::fr const& memo = 999;

    auto private_inputs = do_private_call_get_kernel_inputs_init(false, deposit, { amount, asset_id, memo });

    auto const& contract_address =
        private_inputs.private_call.call_stack_item.public_inputs.call_context.storage_contract_address;

    auto [read_requests, read_request_membership_witnesses, root] = get_random_reads(contract_address, 1);
    private_inputs.private_call.call_stack_item.public_inputs.historic_private_data_tree_root = root;
    private_inputs.private_call.call_stack_item.public_inputs.read_requests = read_requests;
    private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;

    // tweak sibling path so it gives wrong root when paired with its request
    DummyComposer composer = DummyComposer("native_private_kernel_init_tests__native_one_read_requests_works");
    auto const& public_inputs = native_private_kernel_circuit_initial(composer, private_inputs);

    validate_no_new_deployed_contract(public_inputs);

    auto failure = composer.get_first_failure();
    if (failure.code != CircuitErrorCode::NO_ERROR) {
        info("failure: ", failure);
    }
    ASSERT_FALSE(composer.failed());

    // Check the first nullifier is hash of the signed tx request
    ASSERT_EQ(public_inputs.end.new_nullifiers[0], private_inputs.tx_request.hash());
}

TEST_F(native_private_kernel_init_tests, native_two_read_requests_works)
{
    // two read requests should work

    NT::fr const& amount = 5;
    NT::fr const& asset_id = 1;
    NT::fr const& memo = 999;

    auto private_inputs = do_private_call_get_kernel_inputs_init(false, deposit, { amount, asset_id, memo });

    auto const& contract_address =
        private_inputs.private_call.call_stack_item.public_inputs.call_context.storage_contract_address;

    auto [read_requests, read_request_membership_witnesses, root] = get_random_reads(contract_address, 2);
    private_inputs.private_call.call_stack_item.public_inputs.historic_private_data_tree_root = root;
    private_inputs.private_call.call_stack_item.public_inputs.read_requests = read_requests;
    private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;

    DummyComposer composer = DummyComposer("native_private_kernel_init_tests__native_two_read_requests_works");
    auto const& public_inputs = native_private_kernel_circuit_initial(composer, private_inputs);

    validate_no_new_deployed_contract(public_inputs);

    auto failure = composer.get_first_failure();
    if (failure.code != CircuitErrorCode::NO_ERROR) {
        info("failure: ", failure);
    }
    ASSERT_FALSE(composer.failed());

    // Check the first nullifier is hash of the signed tx request
    ASSERT_EQ(public_inputs.end.new_nullifiers[0], private_inputs.tx_request.hash());
}

TEST_F(native_private_kernel_init_tests, native_max_read_requests_works)
{
    // max read requests should work

    NT::fr const& amount = 5;
    NT::fr const& asset_id = 1;
    NT::fr const& memo = 999;

    auto private_inputs = do_private_call_get_kernel_inputs_init(false, deposit, { amount, asset_id, memo });

    auto const& contract_address =
        private_inputs.private_call.call_stack_item.public_inputs.call_context.storage_contract_address;

    auto [read_requests, read_request_membership_witnesses, root] =
        get_random_reads(contract_address, READ_REQUESTS_LENGTH);
    private_inputs.private_call.call_stack_item.public_inputs.historic_private_data_tree_root = root;
    private_inputs.private_call.call_stack_item.public_inputs.read_requests = read_requests;
    private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;

    DummyComposer composer = DummyComposer("native_private_kernel_init_tests__native_max_read_requests_works");
    auto const& public_inputs = native_private_kernel_circuit_initial(composer, private_inputs);

    validate_no_new_deployed_contract(public_inputs);

    auto failure = composer.get_first_failure();
    if (failure.code != CircuitErrorCode::NO_ERROR) {
        info("failure: ", failure);
    }
    ASSERT_FALSE(composer.failed());

    // Check the first nullifier is hash of the signed tx request
    ASSERT_EQ(public_inputs.end.new_nullifiers[0], private_inputs.tx_request.hash());
}

// TODO(dbanks12): more tests of read_requests for multiple iterations.
// Check enforcement that inner iterations' read_requests match root in constants
// https://github.com/AztecProtocol/aztec-packages/issues/786

/**
 **************************************************************
 * Native inner private kernel circuit tests.
 **************************************************************
 */
TEST_F(native_private_kernel_inner_tests, private_function_zero_storage_contract_address_fails)
{
    NT::fr const arg0 = 5;
    NT::fr const arg1 = 1;
    NT::fr const arg2 = 999;

    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, { arg0, arg1, arg2 });

    // Set storage_contract_address to 0
    private_inputs.private_call.call_stack_item.public_inputs.call_context.storage_contract_address = 0;
    private_inputs.private_call.call_stack_item.contract_address = 0;

    // Modify the call stack item's hash with the newly added contract address.
    private_inputs.previous_kernel.public_inputs.end.private_call_stack[0] =
        private_inputs.private_call.call_stack_item.hash();

    // Invoke the native private kernel circuit
    DummyComposer composer =
        DummyComposer("private_kernel_tests__private_function_zero_storage_contract_address_fails");
    native_private_kernel_circuit_inner(composer, private_inputs);

    // Assertion checks
    EXPECT_TRUE(composer.failed());
    EXPECT_EQ(composer.get_first_failure().code, CircuitErrorCode::PRIVATE_KERNEL__INVALID_CONTRACT_ADDRESS);
    EXPECT_EQ(composer.get_first_failure().message,
              "contract address can't be 0 for non-contract deployment related transactions");
}

TEST_F(native_private_kernel_inner_tests, private_function_incorrect_contract_tree_root_fails)
{
    NT::fr const arg0 = 5;
    NT::fr const arg1 = 1;
    NT::fr const arg2 = 999;

    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, { arg0, arg1, arg2 });

    // Set private_historic_tree_roots to a random scalar.
    private_inputs.previous_kernel.public_inputs.constants.historic_tree_roots.private_historic_tree_roots
        .contract_tree_root = NT::fr::random_element();

    // Invoke the native private kernel circuit
    DummyComposer composer = DummyComposer("private_kernel_tests__private_function_incorrect_contract_tree_root_fails");
    native_private_kernel_circuit_inner(composer, private_inputs);

    // Assertion checks
    EXPECT_TRUE(composer.failed());
    EXPECT_EQ(
        composer.get_first_failure().code,
        CircuitErrorCode::PRIVATE_KERNEL__PURPORTED_CONTRACT_TREE_ROOT_AND_PREVIOUS_KERNEL_CONTRACT_TREE_ROOT_MISMATCH);
    EXPECT_EQ(composer.get_first_failure().message,
              "purported_contract_tree_root doesn't match previous_kernel_contract_tree_root");
}

TEST_F(native_private_kernel_inner_tests, private_function_incorrect_contract_leaf_index_fails)
{
    NT::fr const arg0 = 5;
    NT::fr const arg1 = 1;
    NT::fr const arg2 = 999;

    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, { arg0, arg1, arg2 });

    // Set the leaf index of the contract leaf to 20 (the correct value is 1).
    NT::fr const wrong_idx = 20;
    private_inputs.private_call.contract_leaf_membership_witness.leaf_index = wrong_idx;

    // Invoke the native private kernel circuit
    DummyComposer composer =
        DummyComposer("private_kernel_tests__private_function_incorrect_contract_leaf_index_fails");
    native_private_kernel_circuit_inner(composer, private_inputs);

    // Assertion checks
    EXPECT_TRUE(composer.failed());
    EXPECT_EQ(composer.get_first_failure().code,
              CircuitErrorCode::PRIVATE_KERNEL__COMPUTED_CONTRACT_TREE_ROOT_AND_PURPORTED_CONTRACT_TREE_ROOT_MISMATCH);
    EXPECT_EQ(composer.get_first_failure().message,
              "computed_contract_tree_root doesn't match purported_contract_tree_root");
}

TEST_F(native_private_kernel_inner_tests, private_function_incorrect_contract_leaf_sibling_path_fails)
{
    NT::fr const arg0 = 5;
    NT::fr const arg1 = 1;
    NT::fr const arg2 = 999;

    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, { arg0, arg1, arg2 });

    // Change the contract leaf's membership proof.
    private_inputs.private_call.contract_leaf_membership_witness.sibling_path[0] = fr::random_element();

    // Invoke the native private kernel circuit
    DummyComposer composer =
        DummyComposer("private_kernel_tests__private_function_incorrect_contract_leaf_sibling_path_fails");
    native_private_kernel_circuit_inner(composer, private_inputs);

    // Assertion checks
    EXPECT_TRUE(composer.failed());
    EXPECT_EQ(composer.get_first_failure().code,
              CircuitErrorCode::PRIVATE_KERNEL__COMPUTED_CONTRACT_TREE_ROOT_AND_PURPORTED_CONTRACT_TREE_ROOT_MISMATCH);
    EXPECT_EQ(composer.get_first_failure().message,
              "computed_contract_tree_root doesn't match purported_contract_tree_root");
}

TEST_F(native_private_kernel_inner_tests, private_function_incorrect_function_leaf_index_fails)
{
    NT::fr const arg0 = 5;
    NT::fr const arg1 = 1;
    NT::fr const arg2 = 999;

    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, { arg0, arg1, arg2 });

    // Set the leaf index of the function leaf to 10 (the correct value is 1).
    NT::fr const wrong_idx = 10;
    private_inputs.private_call.function_leaf_membership_witness.leaf_index = wrong_idx;

    // Invoke the native private kernel circuit
    DummyComposer composer =
        DummyComposer("private_kernel_tests__private_function_incorrect_contract_leaf_index_fails");
    native_private_kernel_circuit_inner(composer, private_inputs);

    // Assertion checks
    EXPECT_TRUE(composer.failed());
    EXPECT_EQ(composer.get_first_failure().code,
              CircuitErrorCode::PRIVATE_KERNEL__COMPUTED_CONTRACT_TREE_ROOT_AND_PURPORTED_CONTRACT_TREE_ROOT_MISMATCH);
    EXPECT_EQ(composer.get_first_failure().message,
              "computed_contract_tree_root doesn't match purported_contract_tree_root");
}

TEST_F(native_private_kernel_inner_tests, private_function_incorrect_function_leaf_sibling_path_fails)
{
    NT::fr const arg0 = 5;
    NT::fr const arg1 = 1;
    NT::fr const arg2 = 999;

    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, { arg0, arg1, arg2 });

    // Change the function leaf's membership proof.
    private_inputs.private_call.function_leaf_membership_witness.sibling_path[0] = fr::random_element();

    // Invoke the native private kernel circuit
    DummyComposer composer =
        DummyComposer("private_kernel_tests__private_function_incorrect_contract_leaf_sibling_path_fails");
    native_private_kernel_circuit_inner(composer, private_inputs);

    // Assertion checks
    EXPECT_TRUE(composer.failed());
    EXPECT_EQ(composer.get_first_failure().code,
              CircuitErrorCode::PRIVATE_KERNEL__COMPUTED_CONTRACT_TREE_ROOT_AND_PURPORTED_CONTRACT_TREE_ROOT_MISMATCH);
    EXPECT_EQ(composer.get_first_failure().message,
              "computed_contract_tree_root doesn't match purported_contract_tree_root");
}

// TODO(suyash): Disabled until https://github.com/AztecProtocol/aztec-packages/issues/499 is resolved.
TEST_F(native_private_kernel_inner_tests, DISABLED_private_function_incorrect_call_stack_item_hash_fails)
{
    NT::fr const arg0 = 5;
    NT::fr const arg1 = 1;
    NT::fr const arg2 = 999;

    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, { arg0, arg1, arg2 });

    // Set the first call stack member corresponding to the `deposit` function to random scalar.
    private_inputs.private_call.call_stack_item.public_inputs.private_call_stack[0] = NT::fr::random_element();

    // Invoke the native private kernel circuit
    DummyComposer composer =
        DummyComposer("private_kernel_tests__private_function_incorrect_call_stack_item_hash_fails");
    native_private_kernel_circuit_inner(composer, private_inputs);

    // Assertion checks
    EXPECT_TRUE(composer.failed());
    EXPECT_EQ(composer.get_first_failure().code,
              CircuitErrorCode::PRIVATE_KERNEL__CALCULATED_PRIVATE_CALL_HASH_AND_PROVIDED_PRIVATE_CALL_HASH_MISMATCH);
    EXPECT_EQ(composer.get_first_failure().message,
              "calculated private_call_hash does not match provided private_call_hash at the top of the call stack");
}

TEST_F(native_private_kernel_inner_tests, private_kernel_should_fail_if_aggregating_too_many_commitments)
{
    // Negative test to check if push_array_to_array fails if two many commitments are merged together
    DummyComposer composer = DummyComposer("should_fail_if_aggregating_too_many_commitments");

    NT::fr const& amount = 5;
    NT::fr const& asset_id = 1;
    NT::fr const& memo = 999;

    PrivateKernelInputsInner<NT> private_inputs =
        do_private_call_get_kernel_inputs_inner(false, deposit, { amount, asset_id, memo });

    // Mock the previous new commitments to be full, therefore no need commitments can be added
    std::array<fr, KERNEL_NEW_COMMITMENTS_LENGTH> full_new_commitments{};
    for (size_t i = 0; i < KERNEL_NEW_COMMITMENTS_LENGTH; ++i) {
        full_new_commitments[i] = i + 1;
    }
    private_inputs.previous_kernel.public_inputs.end.new_commitments = full_new_commitments;
    auto const& public_inputs = native_private_kernel_circuit_inner(composer, private_inputs);

    ASSERT_TRUE(composer.failed());
    ASSERT_EQ(composer.get_first_failure().code, CircuitErrorCode::ARRAY_OVERFLOW);
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
    NT::fr const& amount = 5;
    NT::fr const& asset_id = 1;
    NT::fr const& memo = 999;

    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, { amount, asset_id, memo });

    auto const& contract_address =
        private_inputs.private_call.call_stack_item.public_inputs.call_context.storage_contract_address;

    auto [read_requests, read_request_membership_witnesses, root] = get_random_reads(contract_address, 2);
    private_inputs.previous_kernel.public_inputs.constants.historic_tree_roots.private_historic_tree_roots
        .private_data_tree_root = root;
    private_inputs.private_call.call_stack_item.public_inputs.historic_private_data_tree_root = root;

    // tweak read_request so it gives wrong root when paired with its sibling path
    read_requests[1] += 1;

    private_inputs.private_call.call_stack_item.public_inputs.read_requests = read_requests;
    private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;

    DummyComposer composer = DummyComposer("native_private_kernel_inner_tests__native_read_request_bad_request");
    auto const& public_inputs = native_private_kernel_circuit_inner(composer, private_inputs);

    validate_no_new_deployed_contract(public_inputs);

    ASSERT(composer.failed());
    ASSERT_EQ(composer.get_first_failure().code,
              CircuitErrorCode::PRIVATE_KERNEL__READ_REQUEST_PRIVATE_DATA_ROOT_MISMATCH);
}

TEST_F(native_private_kernel_inner_tests, native_read_request_bad_leaf_index)
{
    NT::fr const& amount = 5;
    NT::fr const& asset_id = 1;
    NT::fr const& memo = 999;

    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, { amount, asset_id, memo });

    auto const& contract_address =
        private_inputs.private_call.call_stack_item.public_inputs.call_context.storage_contract_address;

    auto [read_requests, read_request_membership_witnesses, root] = get_random_reads(contract_address, 2);
    private_inputs.previous_kernel.public_inputs.constants.historic_tree_roots.private_historic_tree_roots
        .private_data_tree_root = root;
    private_inputs.private_call.call_stack_item.public_inputs.historic_private_data_tree_root = root;

    // tweak leaf index so it gives wrong root when paired with its request and sibling path
    read_request_membership_witnesses[1].leaf_index += 1;
    private_inputs.private_call.call_stack_item.public_inputs.read_requests = read_requests;
    private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;

    DummyComposer composer = DummyComposer("native_private_kernel_inner_tests__native_read_request_bad_leaf_index");
    auto const& public_inputs = native_private_kernel_circuit_inner(composer, private_inputs);

    validate_no_new_deployed_contract(public_inputs);

    ASSERT(composer.failed());
    ASSERT_EQ(composer.get_first_failure().code,
              CircuitErrorCode::PRIVATE_KERNEL__READ_REQUEST_PRIVATE_DATA_ROOT_MISMATCH);
}

TEST_F(native_private_kernel_inner_tests, native_read_request_bad_sibling_path)
{
    NT::fr const& amount = 5;
    NT::fr const& asset_id = 1;
    NT::fr const& memo = 999;

    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, { amount, asset_id, memo });

    auto const& contract_address =
        private_inputs.private_call.call_stack_item.public_inputs.call_context.storage_contract_address;

    auto [read_requests, read_request_membership_witnesses, root] = get_random_reads(contract_address, 2);
    private_inputs.previous_kernel.public_inputs.constants.historic_tree_roots.private_historic_tree_roots
        .private_data_tree_root = root;
    private_inputs.private_call.call_stack_item.public_inputs.historic_private_data_tree_root = root;

    // tweak sibling path so it gives wrong root when paired with its request
    read_request_membership_witnesses[1].sibling_path[1] += 1;
    private_inputs.private_call.call_stack_item.public_inputs.read_requests = read_requests;
    private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;

    DummyComposer composer = DummyComposer("native_private_kernel_inner_tests__native_read_request_bad_sibling_path");
    auto const& public_inputs = native_private_kernel_circuit_inner(composer, private_inputs);

    validate_no_new_deployed_contract(public_inputs);

    ASSERT(composer.failed());
    ASSERT_EQ(composer.get_first_failure().code,
              CircuitErrorCode::PRIVATE_KERNEL__READ_REQUEST_PRIVATE_DATA_ROOT_MISMATCH);
}

TEST_F(native_private_kernel_inner_tests, native_read_request_root_mismatch)
{
    NT::fr const& amount = 5;
    NT::fr const& asset_id = 1;
    NT::fr const& memo = 999;

    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, { amount, asset_id, memo });

    auto const& contract_address =
        private_inputs.private_call.call_stack_item.public_inputs.call_context.storage_contract_address;

    // generate two random sets of read requests and mix them so their roots don't match
    auto [read_requests0, read_request_membership_witnesses0, root] = get_random_reads(contract_address, 2);
    private_inputs.previous_kernel.public_inputs.constants.historic_tree_roots.private_historic_tree_roots
        .private_data_tree_root = root;
    private_inputs.private_call.call_stack_item.public_inputs.historic_private_data_tree_root = root;
    auto [read_requests1, read_request_membership_witnesses1, _root] = get_random_reads(contract_address, 2);
    std::array<NT::fr, READ_REQUESTS_LENGTH> bad_requests{};
    std::array<MembershipWitness<NT, PRIVATE_DATA_TREE_HEIGHT>, READ_REQUESTS_LENGTH> bad_witnesses;
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

    DummyComposer composer = DummyComposer("native_private_kernel_inner_tests__native_read_request_root_mismatch");
    auto const& public_inputs = native_private_kernel_circuit_inner(composer, private_inputs);

    validate_no_new_deployed_contract(public_inputs);

    ASSERT_TRUE(composer.failed());
    ASSERT_EQ(composer.get_first_failure().code,
              CircuitErrorCode::PRIVATE_KERNEL__READ_REQUEST_PRIVATE_DATA_ROOT_MISMATCH);
}

TEST_F(native_private_kernel_inner_tests, native_no_read_requests_works)
{
    // no read requests should work

    NT::fr const& amount = 5;
    NT::fr const& asset_id = 1;
    NT::fr const& memo = 999;

    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, { amount, asset_id, memo });

    // empty requests
    std::array<fr, READ_REQUESTS_LENGTH> const read_requests = zero_array<fr, READ_REQUESTS_LENGTH>();
    std::array<MembershipWitness<NT, PRIVATE_DATA_TREE_HEIGHT>, READ_REQUESTS_LENGTH> const
        read_request_membership_witnesses{};
    private_inputs.private_call.call_stack_item.public_inputs.read_requests = read_requests;
    private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;

    DummyComposer composer = DummyComposer("native_private_kernel_inner_tests__native_no_read_requests_works");
    auto const& public_inputs = native_private_kernel_circuit_inner(composer, private_inputs);

    validate_no_new_deployed_contract(public_inputs);

    auto failure = composer.get_first_failure();
    if (failure.code != CircuitErrorCode::NO_ERROR) {
        info("failure: ", failure);
    }
    ASSERT_FALSE(composer.failed());
}

TEST_F(native_private_kernel_inner_tests, native_one_read_requests_works)
{
    // one read request should work

    NT::fr const& amount = 5;
    NT::fr const& asset_id = 1;
    NT::fr const& memo = 999;

    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, { amount, asset_id, memo });

    auto const& contract_address =
        private_inputs.private_call.call_stack_item.public_inputs.call_context.storage_contract_address;

    auto [read_requests, read_request_membership_witnesses, root] = get_random_reads(contract_address, 1);
    private_inputs.previous_kernel.public_inputs.constants.historic_tree_roots.private_historic_tree_roots
        .private_data_tree_root = root;
    private_inputs.private_call.call_stack_item.public_inputs.historic_private_data_tree_root = root;
    private_inputs.private_call.call_stack_item.public_inputs.read_requests = read_requests;
    private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;

    DummyComposer composer = DummyComposer("native_private_kernel_inner_tests__native_one_read_requests_works");
    auto const& public_inputs = native_private_kernel_circuit_inner(composer, private_inputs);

    validate_no_new_deployed_contract(public_inputs);

    auto failure = composer.get_first_failure();
    if (failure.code != CircuitErrorCode::NO_ERROR) {
        info("failure: ", failure);
    }
    ASSERT_FALSE(composer.failed());
}

TEST_F(native_private_kernel_inner_tests, native_two_read_requests_works)
{
    // two read requests should work

    NT::fr const& amount = 5;
    NT::fr const& asset_id = 1;
    NT::fr const& memo = 999;

    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, { amount, asset_id, memo });

    auto const& contract_address =
        private_inputs.private_call.call_stack_item.public_inputs.call_context.storage_contract_address;

    auto [read_requests, read_request_membership_witnesses, root] = get_random_reads(contract_address, 2);
    private_inputs.previous_kernel.public_inputs.constants.historic_tree_roots.private_historic_tree_roots
        .private_data_tree_root = root;
    private_inputs.private_call.call_stack_item.public_inputs.historic_private_data_tree_root = root;
    private_inputs.private_call.call_stack_item.public_inputs.read_requests = read_requests;
    private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;

    DummyComposer composer = DummyComposer("native_private_kernel_inner_tests__native_two_read_requests_works");
    auto const& public_inputs = native_private_kernel_circuit_inner(composer, private_inputs);

    validate_no_new_deployed_contract(public_inputs);

    auto failure = composer.get_first_failure();
    if (failure.code != CircuitErrorCode::NO_ERROR) {
        info("failure: ", failure);
    }
    ASSERT_FALSE(composer.failed());
}

TEST_F(native_private_kernel_inner_tests, native_max_read_requests_works)
{
    // max read requests should work

    NT::fr const& amount = 5;
    NT::fr const& asset_id = 1;
    NT::fr const& memo = 999;

    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, { amount, asset_id, memo });

    auto const& contract_address =
        private_inputs.private_call.call_stack_item.public_inputs.call_context.storage_contract_address;

    auto [read_requests, read_request_membership_witnesses, root] =
        get_random_reads(contract_address, READ_REQUESTS_LENGTH);
    private_inputs.previous_kernel.public_inputs.constants.historic_tree_roots.private_historic_tree_roots
        .private_data_tree_root = root;
    private_inputs.private_call.call_stack_item.public_inputs.historic_private_data_tree_root = root;
    private_inputs.private_call.call_stack_item.public_inputs.read_requests = read_requests;
    private_inputs.private_call.read_request_membership_witnesses = read_request_membership_witnesses;

    DummyComposer composer = DummyComposer("native_private_kernel_inner_tests__native_max_read_requests_works");
    auto const& public_inputs = native_private_kernel_circuit_inner(composer, private_inputs);

    validate_no_new_deployed_contract(public_inputs);

    auto failure = composer.get_first_failure();
    if (failure.code != CircuitErrorCode::NO_ERROR) {
        info("failure: ", failure);
    }
    ASSERT_FALSE(composer.failed());
}

}  // namespace aztec3::circuits::kernel::private_kernel
