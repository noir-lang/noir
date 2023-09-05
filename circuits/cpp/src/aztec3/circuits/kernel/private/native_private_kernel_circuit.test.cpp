#include "testing_harness.hpp"

#include "aztec3/circuits/abis/private_kernel/private_kernel_inputs_init.hpp"
#include "aztec3/circuits/abis/private_kernel/private_kernel_inputs_ordering.hpp"
#include "aztec3/circuits/abis/read_request_membership_witness.hpp"
#include "aztec3/circuits/apps/test_apps/escrow/deposit.hpp"
#include "aztec3/circuits/hash.hpp"
#include "aztec3/circuits/kernel/private/common.hpp"
#include "aztec3/circuits/kernel/private/init.hpp"
#include "aztec3/constants.hpp"
#include "aztec3/utils/circuit_errors.hpp"

#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include <barretenberg/barretenberg.hpp>

#include <gtest/gtest.h>

#include <array>
#include <cstdint>

namespace aztec3::circuits::kernel::private_kernel {

using aztec3::circuits::apps::test_apps::escrow::deposit;

using abis::private_kernel::PrivateKernelInputsOrdering;
using aztec3::circuits::kernel::private_kernel::testing_harness::do_private_call_get_kernel_inputs_init;
using aztec3::circuits::kernel::private_kernel::testing_harness::do_private_call_get_kernel_inputs_inner;
using aztec3::utils::array_length;
using aztec3::utils::CircuitErrorCode;


// TODO(https://github.com/AztecProtocol/aztec-packages/issues/892): test expected kernel failures if transient
// reads (or their hints) don't match
// TODO(https://github.com/AztecProtocol/aztec-packages/issues/836): test expected kernel failures if nullifiers (or
// their hints) don't match

/**************************************************************
 * MULTI ITERATION UNIT TESTS FOR NATIVE PRIVATE KERNEL CIRCUIT
 **************************************************************/


// NOTE: *DO NOT* call fr constructors in static initializers and assign them to constants. This will fail. Instead,
// use lazy initialization or functions. Lambdas were introduced here. amount = 5,  asset_id = 1, memo = 999
const auto standard_test_args = [] { return std::vector<NT::fr>{ NT::fr(5), NT::fr(1), NT::fr(999) }; };
class native_private_kernel_tests : public ::testing::Test {
  protected:
    static void SetUpTestSuite() { barretenberg::srs::init_crs_factory("../barretenberg/cpp/srs_db/ignition"); }
};


// 1. We send transient read request on value 23 and pending commitment 12
// 2. We send transient read request on value 12 and pending commitment 23
// We expect both read requests and commitments to be successfully matched in ordering circuit.
TEST_F(native_private_kernel_tests, native_accumulate_transient_read_requests)
{
    auto private_inputs_init = do_private_call_get_kernel_inputs_init(false, deposit, standard_test_args());

    private_inputs_init.private_call.call_stack_item.public_inputs.new_commitments[0] = fr(12);
    private_inputs_init.private_call.call_stack_item.public_inputs.read_requests[0] = fr(23);
    private_inputs_init.private_call.read_request_membership_witnesses[0].is_transient = true;

    std::array<fr, MAX_READ_REQUESTS_PER_TX> hint_to_commitments{};
    hint_to_commitments[0] = fr(1);

    DummyBuilder builder = DummyBuilder("native_private_kernel_tests__native_accumulate_transient_read_requests");
    auto public_inputs = native_private_kernel_circuit_initial(builder, private_inputs_init);

    ASSERT_FALSE(builder.failed()) << "failure: " << builder.get_first_failure()
                                   << " with code: " << builder.get_first_failure().code;
    ASSERT_TRUE(array_length(public_inputs.end.new_commitments) == 1);
    ASSERT_TRUE(array_length(public_inputs.end.read_requests) == 1);

    auto private_inputs_inner = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    private_inputs_inner.private_call.call_stack_item.public_inputs.new_commitments[0] = fr(23);
    private_inputs_inner.private_call.call_stack_item.public_inputs.read_requests[0] = fr(12);
    private_inputs_inner.private_call.read_request_membership_witnesses[0].is_transient = true;

    hint_to_commitments[1] = fr(0);

    // We need to update the previous_kernel's private_call_stack because the current_call_stack_item has changed
    // i.e. we changed the new_commitments and read_requests of the current_call_stack_item's public_inputs
    private_inputs_inner.previous_kernel.public_inputs.end.private_call_stack[0] =
        private_inputs_inner.private_call.call_stack_item.hash();

    // The original call is not multi-iterative (call stack depth == 1) and we re-feed the same private call stack
    public_inputs.end.private_call_stack = private_inputs_inner.previous_kernel.public_inputs.end.private_call_stack;
    private_inputs_inner.previous_kernel.public_inputs = public_inputs;

    public_inputs = native_private_kernel_circuit_inner(builder, private_inputs_inner);

    ASSERT_FALSE(builder.failed()) << "failure: " << builder.get_first_failure()
                                   << " with code: " << builder.get_first_failure().code;
    ASSERT_TRUE(array_length(public_inputs.end.new_commitments) == 2);
    ASSERT_TRUE(array_length(public_inputs.end.read_requests) == 2);

    auto& previous_kernel = private_inputs_inner.previous_kernel;
    previous_kernel.public_inputs = public_inputs;

    PrivateKernelInputsOrdering<NT> private_inputs{ previous_kernel, hint_to_commitments };
    auto final_public_inputs = native_private_kernel_circuit_ordering(builder, private_inputs);

    ASSERT_FALSE(builder.failed()) << "failure: " << builder.get_first_failure()
                                   << " with code: " << builder.get_first_failure().code;
    ASSERT_TRUE(array_length(final_public_inputs.end.new_commitments) == 2);  // no commitments squashed
}

// 1. We send transient read request on value 23 and pending commitment 10
// 2. We send transient read request on value 12 and pending commitment 23
// We expect the read request on value 12 to fail as there is no corresponding pending commitment.
TEST_F(native_private_kernel_tests, native_transient_read_requests_no_match)
{
    auto private_inputs_init = do_private_call_get_kernel_inputs_init(false, deposit, standard_test_args());

    private_inputs_init.private_call.call_stack_item.public_inputs.new_commitments[0] = fr(10);
    private_inputs_init.private_call.call_stack_item.public_inputs.read_requests[0] = fr(23);
    private_inputs_init.private_call.read_request_membership_witnesses[0].is_transient = true;

    std::array<fr, MAX_READ_REQUESTS_PER_TX> hint_to_commitments{};
    hint_to_commitments[0] = fr(1);

    DummyBuilder builder = DummyBuilder("native_private_kernel_tests__native_transient_read_requests_no_match");
    auto public_inputs = native_private_kernel_circuit_initial(builder, private_inputs_init);

    ASSERT_FALSE(builder.failed()) << "failure: " << builder.get_first_failure()
                                   << " with code: " << builder.get_first_failure().code;
    ASSERT_TRUE(array_length(public_inputs.end.new_commitments) == 1);
    ASSERT_TRUE(array_length(public_inputs.end.read_requests) == 1);

    auto private_inputs_inner = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    private_inputs_inner.private_call.call_stack_item.public_inputs.new_commitments[0] = fr(23);
    private_inputs_inner.private_call.call_stack_item.public_inputs.read_requests[0] = fr(12);
    private_inputs_inner.private_call.read_request_membership_witnesses[0].is_transient = true;

    hint_to_commitments[1] = fr(0);  // There is not correct possible value.

    // We need to update the previous_kernel's private_call_stack because the current_call_stack_item has changed
    // i.e. we changed the new_commitments and read_requests of the current_call_stack_item's public_inputs
    private_inputs_inner.previous_kernel.public_inputs.end.private_call_stack[0] =
        private_inputs_inner.private_call.call_stack_item.hash();

    // The original call is not multi-iterative (call stack depth == 1) and we re-feed the same private call stack
    public_inputs.end.private_call_stack = private_inputs_inner.previous_kernel.public_inputs.end.private_call_stack;
    private_inputs_inner.previous_kernel.public_inputs = public_inputs;

    public_inputs = native_private_kernel_circuit_inner(builder, private_inputs_inner);

    ASSERT_FALSE(builder.failed()) << "failure: " << builder.get_first_failure()
                                   << " with code: " << builder.get_first_failure().code;
    ASSERT_TRUE(array_length(public_inputs.end.new_commitments) == 2);
    ASSERT_TRUE(array_length(public_inputs.end.read_requests) == 2);

    auto& previous_kernel = private_inputs_inner.previous_kernel;
    previous_kernel.public_inputs = public_inputs;

    PrivateKernelInputsOrdering<NT> private_inputs{ previous_kernel, hint_to_commitments };
    auto final_public_inputs = native_private_kernel_circuit_ordering(builder, private_inputs);

    ASSERT_TRUE(builder.failed());
    ASSERT_TRUE(builder.get_first_failure().code == CircuitErrorCode::PRIVATE_KERNEL__TRANSIENT_READ_REQUEST_NO_MATCH);

    ASSERT_TRUE(array_length(final_public_inputs.end.new_commitments) == 2);  // no commitments squashed
}

// Testing that the special value EMPTY_NULLIFIED_COMMITMENT keeps new_nullifiers aligned with nullified_commitments.
TEST_F(native_private_kernel_tests, native_empty_nullified_commitment_respected)
{
    auto private_inputs_inner = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    private_inputs_inner.private_call.call_stack_item.public_inputs.new_commitments[0] = fr(23);
    private_inputs_inner.private_call.call_stack_item.public_inputs.new_commitments[1] = fr(33);

    private_inputs_inner.private_call.call_stack_item.public_inputs.new_nullifiers[0] = fr(11);
    private_inputs_inner.private_call.call_stack_item.public_inputs.new_nullifiers[1] = fr(18);

    private_inputs_inner.private_call.call_stack_item.public_inputs.nullified_commitments[0] =
        fr(EMPTY_NULLIFIED_COMMITMENT);
    private_inputs_inner.private_call.call_stack_item.public_inputs.nullified_commitments[1] = fr(23);

    // update the private call stack contents to reflect the above changes which affect the item hash
    private_inputs_inner.previous_kernel.public_inputs.end.private_call_stack[0] =
        private_inputs_inner.private_call.call_stack_item.hash();

    DummyBuilder builder = DummyBuilder("native_private_kernel_tests__native_empty_nullified_commitment_respected");

    auto public_inputs = native_private_kernel_circuit_inner(builder, private_inputs_inner);

    ASSERT_FALSE(builder.failed()) << "failure: " << builder.get_first_failure()
                                   << " with code: " << builder.get_first_failure().code;

    // EMPTY nullified commitment should keep new_nullifiers aligned with nullified_commitments
    ASSERT_TRUE(public_inputs.end.nullified_commitments[0] == fr(EMPTY_NULLIFIED_COMMITMENT));
    ASSERT_TRUE(public_inputs.end.nullified_commitments[1] != fr(0) &&
                public_inputs.end.nullified_commitments[1] != fr(EMPTY_NULLIFIED_COMMITMENT));

    // Nothing squashed yet (until ordering circuit)
    ASSERT_TRUE(array_length(public_inputs.end.new_nullifiers) == 2);
    ASSERT_TRUE(array_length(public_inputs.end.new_commitments) == 2);
    // explicitly EMPTY commitment respected in array
    ASSERT_TRUE(array_length(public_inputs.end.nullified_commitments) == 2);

    auto& previous_kernel = private_inputs_inner.previous_kernel;
    previous_kernel.public_inputs = public_inputs;

    PrivateKernelInputsOrdering<NT> private_inputs{ .previous_kernel = previous_kernel };

    auto final_public_inputs = native_private_kernel_circuit_ordering(builder, private_inputs);

    ASSERT_FALSE(builder.failed()) << "failure: " << builder.get_first_failure()
                                   << " with code: " << builder.get_first_failure().code;

    ASSERT_TRUE(array_length(final_public_inputs.end.new_commitments) == 1);  // 1/2 commitment squashed
    ASSERT_TRUE(array_length(final_public_inputs.end.new_nullifiers) == 1);   // 1/2 nullifier squashed
}

}  // namespace aztec3::circuits::kernel::private_kernel
