#include "testing_harness.hpp"

#include "aztec3/circuits/apps/test_apps/escrow/deposit.hpp"
#include "aztec3/constants.hpp"
#include "aztec3/utils/circuit_errors.hpp"

#include "barretenberg/ecc/curves/bn254/fr.hpp"
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

TEST_F(native_private_kernel_ordering_tests, native_matching_one_read_request_to_commitment_works)
{
    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    std::array<fr, MAX_NEW_COMMITMENTS_PER_TX> new_commitments{};
    std::array<fr, MAX_READ_REQUESTS_PER_TX> read_requests{};
    std::array<ReadRequestMembershipWitness<NT, PRIVATE_DATA_TREE_HEIGHT>, MAX_READ_REQUESTS_PER_TX>
        read_request_membership_witnesses{};

    new_commitments[0] = fr(1282);
    read_requests[0] = fr(1282);
    read_request_membership_witnesses[0].is_transient = true;

    private_inputs.previous_kernel.public_inputs.end.new_commitments = new_commitments;
    private_inputs.previous_kernel.public_inputs.end.read_requests = read_requests;
    private_inputs.previous_kernel.public_inputs.end.read_request_membership_witnesses =
        read_request_membership_witnesses;


    DummyBuilder builder =
        DummyBuilder("native_private_kernel_ordering_tests__native_matching_one_read_request_to_commitment_works");
    auto const& public_inputs = native_private_kernel_circuit_ordering(builder, private_inputs.previous_kernel);

    auto failure = builder.get_first_failure();
    if (failure.code != CircuitErrorCode::NO_ERROR) {
        info("failure: ", failure);
    }
    ASSERT_FALSE(builder.failed());
    ASSERT_TRUE(array_length(public_inputs.end.new_commitments) == 1);
    ASSERT_TRUE(public_inputs.end.new_commitments[0] == fr(1282));
    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1074): read_request*s
    // can be removed from final public inputs
    ASSERT_TRUE(array_length(public_inputs.end.read_requests) == 0);
    ASSERT_TRUE(array_length(public_inputs.end.read_request_membership_witnesses) == 0);
}

TEST_F(native_private_kernel_ordering_tests, native_matching_some_read_requests_to_commitments_works)
{
    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    std::array<fr, MAX_NEW_COMMITMENTS_PER_TX> new_commitments{};
    std::array<fr, MAX_READ_REQUESTS_PER_TX> read_requests{};
    std::array<ReadRequestMembershipWitness<NT, PRIVATE_DATA_TREE_HEIGHT>, MAX_READ_REQUESTS_PER_TX>
        read_request_membership_witnesses{};

    new_commitments[0] = fr(1285);
    new_commitments[1] = fr(1283);
    new_commitments[2] = fr(1282);
    new_commitments[3] = fr(1284);


    read_requests[0] = fr(1283);
    read_requests[1] = fr(1284);
    read_request_membership_witnesses[0].is_transient = true;
    read_request_membership_witnesses[1].is_transient = true;

    private_inputs.previous_kernel.public_inputs.end.new_commitments = new_commitments;
    private_inputs.previous_kernel.public_inputs.end.read_requests = read_requests;
    private_inputs.previous_kernel.public_inputs.end.read_request_membership_witnesses =
        read_request_membership_witnesses;

    DummyBuilder builder =
        DummyBuilder("native_private_kernel_ordering_tests__native_matching_some_read_requests_to_commitments_works");
    auto const& public_inputs = native_private_kernel_circuit_ordering(builder, private_inputs.previous_kernel);

    auto failure = builder.get_first_failure();
    if (failure.code != CircuitErrorCode::NO_ERROR) {
        info("failure: ", failure);
    }
    ASSERT_FALSE(builder.failed());
    ASSERT_TRUE(array_length(public_inputs.end.new_commitments) == 4);
    ASSERT_TRUE(public_inputs.end.new_commitments[0] == fr(1285));
    ASSERT_TRUE(public_inputs.end.new_commitments[1] == fr(1283));
    ASSERT_TRUE(public_inputs.end.new_commitments[2] == fr(1282));
    ASSERT_TRUE(public_inputs.end.new_commitments[3] == fr(1284));
    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1074): read_request*s
    // can be removed from final public inputs
    ASSERT_TRUE(array_length(public_inputs.end.read_requests) == 0);
    ASSERT_TRUE(array_length(public_inputs.end.read_request_membership_witnesses) == 0);
}

TEST_F(native_private_kernel_ordering_tests, native_read_request_unknown_fails)
{
    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    std::array<fr, MAX_NEW_COMMITMENTS_PER_TX> new_commitments{};
    std::array<fr, MAX_READ_REQUESTS_PER_TX> read_requests{};
    std::array<ReadRequestMembershipWitness<NT, PRIVATE_DATA_TREE_HEIGHT>, MAX_READ_REQUESTS_PER_TX>
        read_request_membership_witnesses{};

    new_commitments[0] = fr(1285);
    new_commitments[1] = fr(1283);
    new_commitments[2] = fr(1282);
    new_commitments[3] = fr(1284);


    read_requests[0] = fr(1284);
    read_requests[1] = fr(1282);
    read_requests[2] = fr(1283);
    read_requests[3] = fr(1286);
    read_request_membership_witnesses[0].is_transient = true;
    read_request_membership_witnesses[1].is_transient = true;
    read_request_membership_witnesses[2].is_transient = true;
    read_request_membership_witnesses[3].is_transient = true;

    private_inputs.previous_kernel.public_inputs.end.new_commitments = new_commitments;
    private_inputs.previous_kernel.public_inputs.end.read_requests = read_requests;
    private_inputs.previous_kernel.public_inputs.end.read_request_membership_witnesses =
        read_request_membership_witnesses;


    DummyBuilder builder = DummyBuilder("native_private_kernel_ordering_tests__native_read_request_unknown_fails");
    native_private_kernel_circuit_ordering(builder, private_inputs.previous_kernel);

    auto failure = builder.get_first_failure();
    ASSERT_EQ(failure.code, CircuitErrorCode::PRIVATE_KERNEL__TRANSIENT_READ_REQUEST_NO_MATCH);
}

TEST_F(native_private_kernel_ordering_tests, native_unresolved_non_transient_read_fails)
{
    auto private_inputs = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    std::array<fr, MAX_NEW_COMMITMENTS_PER_TX> new_commitments{};
    std::array<fr, MAX_READ_REQUESTS_PER_TX> read_requests{};
    std::array<ReadRequestMembershipWitness<NT, PRIVATE_DATA_TREE_HEIGHT>, MAX_READ_REQUESTS_PER_TX>
        read_request_membership_witnesses{};

    new_commitments[0] = fr(1285);


    read_requests[0] = fr(1285);
    read_request_membership_witnesses[0].is_transient = false;  // ordering circuit only allows transient reads

    private_inputs.previous_kernel.public_inputs.end.new_commitments = new_commitments;
    private_inputs.previous_kernel.public_inputs.end.read_requests = read_requests;
    private_inputs.previous_kernel.public_inputs.end.read_request_membership_witnesses =
        read_request_membership_witnesses;


    DummyBuilder builder =
        DummyBuilder("native_private_kernel_ordering_tests__native_unresolved_non_transient_read_fails");
    native_private_kernel_circuit_ordering(builder, private_inputs.previous_kernel);

    auto failure = builder.get_first_failure();
    ASSERT_EQ(failure.code, CircuitErrorCode::PRIVATE_KERNEL__UNRESOLVED_NON_TRANSIENT_READ_REQUEST);
}

}  // namespace aztec3::circuits::kernel::private_kernel
