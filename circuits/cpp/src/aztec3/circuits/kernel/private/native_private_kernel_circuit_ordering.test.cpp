#include "testing_harness.hpp"

#include "aztec3/circuits/apps/test_apps/escrow/deposit.hpp"
#include "aztec3/utils/circuit_errors.hpp"

#include <barretenberg/barretenberg.hpp>

#include <gtest/gtest.h>

#include <array>
#include <cstdint>

namespace aztec3::circuits::kernel::private_kernel {

using aztec3::circuits::apps::test_apps::escrow::deposit;

using aztec3::circuits::kernel::private_kernel::testing_harness::do_private_call_get_kernel_inputs_inner;
using aztec3::utils::array_length;
using aztec3::utils::CircuitErrorCode;

// NOTE: *DO NOT* call fr constructors in static initializers and assign them to constants. This will fail. Instead, use
// lazy initialization or functions. Lambdas were introduced here.
// amount = 5,  asset_id = 1, memo = 999
const auto standard_test_args = [] { return std::vector<NT::fr>{ NT::fr(5), NT::fr(1), NT::fr(999) }; };
class native_private_kernel_ordering_tests : public ::testing::Test {
  protected:
    static void SetUpTestSuite() { barretenberg::srs::init_crs_factory("../barretenberg/cpp/srs_db/ignition"); }
};

TEST_F(native_private_kernel_ordering_tests, native_one_read_request_choping_commitment_works)
{
    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    auto new_commitments = zero_array<fr, KERNEL_NEW_COMMITMENTS_LENGTH>();
    auto read_requests = zero_array<fr, READ_REQUESTS_LENGTH>();
    std::array<MembershipWitness<NT, PRIVATE_DATA_TREE_HEIGHT>, READ_REQUESTS_LENGTH>
        read_request_membership_witnesses{};

    new_commitments[0] = fr(1282);
    read_requests[0] = fr(1282);
    read_request_membership_witnesses[0].leaf_index = fr(-1);

    private_inputs.previous_kernel.public_inputs.end.new_commitments = new_commitments;

    DummyBuilder builder =
        DummyBuilder("native_private_kernel_ordering_tests__native_one_read_request_choping_commitment_works");
    auto const& public_inputs = native_private_kernel_circuit_ordering(
        builder, private_inputs.previous_kernel, read_requests, read_request_membership_witnesses);

    auto failure = builder.get_first_failure();
    if (failure.code != CircuitErrorCode::NO_ERROR) {
        info("failure: ", failure);
    }
    ASSERT_FALSE(builder.failed());
    ASSERT_TRUE(array_length(public_inputs.end.new_commitments) == 0);
}

TEST_F(native_private_kernel_ordering_tests, native_read_requests_choping_commitment_works)
{
    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    auto new_commitments = zero_array<fr, KERNEL_NEW_COMMITMENTS_LENGTH>();
    auto read_requests = zero_array<fr, READ_REQUESTS_LENGTH>();
    std::array<MembershipWitness<NT, PRIVATE_DATA_TREE_HEIGHT>, READ_REQUESTS_LENGTH>
        read_request_membership_witnesses{};

    new_commitments[0] = fr(1285);
    new_commitments[1] = fr(1283);
    new_commitments[2] = fr(1282);
    new_commitments[3] = fr(1284);


    read_requests[0] = fr(1283);
    read_requests[1] = fr(1284);
    read_request_membership_witnesses[0].leaf_index = fr(-1);
    read_request_membership_witnesses[1].leaf_index = fr(-1);

    private_inputs.previous_kernel.public_inputs.end.new_commitments = new_commitments;

    DummyBuilder builder =
        DummyBuilder("native_private_kernel_ordering_tests__native_read_requests_choping_commitment_works");
    auto const& public_inputs = native_private_kernel_circuit_ordering(
        builder, private_inputs.previous_kernel, read_requests, read_request_membership_witnesses);

    auto failure = builder.get_first_failure();
    if (failure.code != CircuitErrorCode::NO_ERROR) {
        info("failure: ", failure);
    }
    ASSERT_FALSE(builder.failed());
    ASSERT_TRUE(array_length(public_inputs.end.new_commitments) == 2);
    ASSERT_TRUE(public_inputs.end.new_commitments[0] == fr(1285));
    ASSERT_TRUE(public_inputs.end.new_commitments[1] == fr(1282));
}

TEST_F(native_private_kernel_ordering_tests, native_read_request_unknown_fails)
{
    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    auto new_commitments = zero_array<fr, KERNEL_NEW_COMMITMENTS_LENGTH>();
    auto read_requests = zero_array<fr, READ_REQUESTS_LENGTH>();
    std::array<MembershipWitness<NT, PRIVATE_DATA_TREE_HEIGHT>, READ_REQUESTS_LENGTH>
        read_request_membership_witnesses{};

    new_commitments[0] = fr(1285);
    new_commitments[1] = fr(1283);
    new_commitments[2] = fr(1282);
    new_commitments[3] = fr(1284);


    read_requests[0] = fr(1284);
    read_requests[1] = fr(1282);
    read_requests[2] = fr(1283);
    read_requests[3] = fr(1286);
    read_request_membership_witnesses[0].leaf_index = fr(-1);
    read_request_membership_witnesses[1].leaf_index = fr(-1);
    read_request_membership_witnesses[2].leaf_index = fr(-1);
    read_request_membership_witnesses[3].leaf_index = fr(-1);

    private_inputs.previous_kernel.public_inputs.end.new_commitments = new_commitments;

    DummyBuilder builder = DummyBuilder("native_private_kernel_ordering_tests__native_read_request_unknown_fails");
    auto const& public_inputs = native_private_kernel_circuit_ordering(
        builder, private_inputs.previous_kernel, read_requests, read_request_membership_witnesses);

    auto failure = builder.get_first_failure();
    ASSERT_EQ(failure.code, CircuitErrorCode::PRIVATE_KERNEL__TRANSIENT_READ_REQUEST_NO_MATCH);
}

}  // namespace aztec3::circuits::kernel::private_kernel
