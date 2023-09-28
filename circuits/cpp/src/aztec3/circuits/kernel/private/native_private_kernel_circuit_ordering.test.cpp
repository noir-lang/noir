#include "testing_harness.hpp"

#include "aztec3/circuits/apps/test_apps/escrow/deposit.hpp"
#include "aztec3/circuits/kernel/private/common.hpp"
#include "aztec3/constants.hpp"
#include "aztec3/utils/array.hpp"
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

// TODO(1998): testing cbind calls private_kernel__sim_ordering, private_kernel__sim_init, private_kernel__sim_inner
//       in their respective test suites once msgpack capabilities allow it. One current limitation is due to
//       the lack of support to deserialize std::variant in particular CircuitResult type.
//       See https://github.com/AztecProtocol/aztec-packages/issues/1998
/**
 * @brief Test cbind
 */
// TEST_F(native_private_kernel_ordering_tests, cbind_private_kernel__sim_ordering)
// {
//     auto func = [](PrivateKernelInputsOrdering<NT> private_inputs) {
//         DummyCircuitBuilder builder = DummyCircuitBuilder("private_kernel__sim_ordering");
//         auto const& public_inputs = native_private_kernel_circuit_ordering(builder, private_inputs);
//         return builder.result_or_error(public_inputs);
//     };

//     NT::fr const& amount = 5;
//     NT::fr const& asset_id = 1;
//     NT::fr const& memo = 999;
//     std::array<NT::fr, NUM_FIELDS_PER_SHA256> const& empty_logs_hash = { NT::fr(16), NT::fr(69) };
//     NT::fr const& empty_log_preimages_length = NT::fr(100);

//     // Generate private inputs including proofs and vkeys for app circuit and previous kernel
//     auto const& private_inputs_inner = do_private_call_get_kernel_inputs_inner(false,
//                                                                                deposit,
//                                                                                { amount, asset_id, memo },
//                                                                                empty_logs_hash,
//                                                                                empty_logs_hash,
//                                                                                empty_log_preimages_length,
//                                                                                empty_log_preimages_length,
//                                                                                empty_logs_hash,
//                                                                                empty_logs_hash,
//                                                                                empty_log_preimages_length,
//                                                                                empty_log_preimages_length,
//                                                                                true);
//     PrivateKernelInputsOrdering<NT> private_inputs{ private_inputs_inner.previous_kernel,
//                                                     std::array<fr, MAX_READ_REQUESTS_PER_TX>{ fr(123), fr(89) } };

//     auto [actual, expected] = call_func_and_wrapper(func, private_kernel__sim_ordering, private_inputs);

//     std::stringstream actual_ss;
//     std::stringstream expected_ss;
//     actual_ss << actual;
//     expected_ss << expected;
//     EXPECT_EQ(actual_ss.str(), expected_ss.str());
// }

TEST_F(native_private_kernel_ordering_tests, native_matching_one_read_request_to_commitment_works)
{
    auto private_inputs_inner = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    std::array<fr, MAX_NEW_NULLIFIERS_PER_TX> new_nullifiers{};
    std::array<fr, MAX_NEW_COMMITMENTS_PER_TX> siloed_commitments{};
    std::array<fr, MAX_NEW_COMMITMENTS_PER_TX> unique_siloed_commitments{};
    std::array<fr, MAX_READ_REQUESTS_PER_TX> read_requests{};
    std::array<fr, MAX_READ_REQUESTS_PER_TX> hints{};

    std::array<ReadRequestMembershipWitness<NT, PRIVATE_DATA_TREE_HEIGHT>, MAX_READ_REQUESTS_PER_TX>
        read_request_membership_witnesses{};

    new_nullifiers[0] = NT::fr::random_element();
    siloed_commitments[0] = NT::fr::random_element();  // create random commitment
    // ordering circuit applies nonces to commitments
    const auto nonce = compute_commitment_nonce<NT>(new_nullifiers[0], 0);
    unique_siloed_commitments[0] =
        siloed_commitments[0] == 0 ? 0 : compute_unique_commitment<NT>(nonce, siloed_commitments[0]);

    read_requests[0] = siloed_commitments[0];
    // hints[0] == fr(0) due to the default initialization of hints
    read_request_membership_witnesses[0].is_transient = true;

    auto& previous_kernel = private_inputs_inner.previous_kernel;

    previous_kernel.public_inputs.end.new_nullifiers = new_nullifiers;
    previous_kernel.public_inputs.end.new_commitments = siloed_commitments;
    previous_kernel.public_inputs.end.read_requests = read_requests;

    PrivateKernelInputsOrdering<NT> private_inputs{ previous_kernel, hints };

    DummyBuilder builder =
        DummyBuilder("native_private_kernel_ordering_tests__native_matching_one_read_request_to_commitment_works");
    auto const& public_inputs = native_private_kernel_circuit_ordering(builder, private_inputs);

    ASSERT_FALSE(builder.failed()) << "failure: " << builder.get_first_failure();
    ASSERT_TRUE(array_length(public_inputs.end.new_commitments) == 1);
    ASSERT_TRUE(public_inputs.end.new_commitments[0] == unique_siloed_commitments[0]);
}

TEST_F(native_private_kernel_ordering_tests, native_matching_some_read_requests_to_commitments_works)
{
    auto private_inputs_inner = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    std::array<fr, MAX_NEW_NULLIFIERS_PER_TX> new_nullifiers{};
    std::array<fr, MAX_NEW_COMMITMENTS_PER_TX> siloed_commitments{};
    std::array<fr, MAX_NEW_COMMITMENTS_PER_TX> unique_siloed_commitments{};
    std::array<fr, MAX_READ_REQUESTS_PER_TX> read_requests{};
    std::array<fr, MAX_READ_REQUESTS_PER_TX> hints{};

    std::array<ReadRequestMembershipWitness<NT, PRIVATE_DATA_TREE_HEIGHT>, MAX_READ_REQUESTS_PER_TX>
        read_request_membership_witnesses{};

    new_nullifiers[0] = NT::fr::random_element();
    const auto& first_nullifier = new_nullifiers[0];
    // create random commitments to input to ordering circuit, and compute their "unique" versions
    // to be expected at the output
    for (size_t c_idx = 0; c_idx < MAX_NEW_COMMITMENTS_PER_TX; c_idx++) {
        siloed_commitments[c_idx] = NT::fr::random_element();  // create random commitment
        // ordering circuit applies nonces to commitments
        const auto nonce = compute_commitment_nonce<NT>(first_nullifier, c_idx);
        unique_siloed_commitments[c_idx] =
            siloed_commitments[c_idx] == 0 ? 0 : compute_unique_commitment<NT>(nonce, siloed_commitments[c_idx]);
    }

    read_requests[0] = siloed_commitments[1];
    read_requests[1] = siloed_commitments[3];
    read_request_membership_witnesses[0].is_transient = true;
    read_request_membership_witnesses[1].is_transient = true;
    hints[0] = fr(1);
    hints[1] = fr(3);

    auto& previous_kernel = private_inputs_inner.previous_kernel;

    previous_kernel.public_inputs.end.new_nullifiers = new_nullifiers;
    previous_kernel.public_inputs.end.new_commitments = siloed_commitments;
    previous_kernel.public_inputs.end.read_requests = read_requests;

    PrivateKernelInputsOrdering<NT> private_inputs{ previous_kernel, hints };

    DummyBuilder builder =
        DummyBuilder("native_private_kernel_ordering_tests__native_matching_some_read_requests_to_commitments_works");
    auto const& public_inputs = native_private_kernel_circuit_ordering(builder, private_inputs);

    ASSERT_FALSE(builder.failed()) << "failure: " << builder.get_first_failure();
    ASSERT_TRUE(array_length(public_inputs.end.new_commitments) == MAX_NEW_COMMITMENTS_PER_TX);
    // ensure that commitments had nonce applied properly and all appear at output
    for (size_t c_idx = 0; c_idx < MAX_NEW_COMMITMENTS_PER_TX; c_idx++) {
        ASSERT_TRUE(public_inputs.end.new_commitments[c_idx] == unique_siloed_commitments[c_idx]);
    }
}

TEST_F(native_private_kernel_ordering_tests, native_read_request_unknown_fails)
{
    auto private_inputs_inner = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    std::array<fr, MAX_NEW_COMMITMENTS_PER_TX> siloed_commitments{};
    std::array<fr, MAX_READ_REQUESTS_PER_TX> read_requests{};
    std::array<fr, MAX_READ_REQUESTS_PER_TX> hints{};
    std::array<ReadRequestMembershipWitness<NT, PRIVATE_DATA_TREE_HEIGHT>, MAX_READ_REQUESTS_PER_TX>
        read_request_membership_witnesses{};

    for (size_t c_idx = 0; c_idx < MAX_NEW_COMMITMENTS_PER_TX; c_idx++) {
        siloed_commitments[c_idx] = NT::fr::random_element();          // create random commitment
        read_requests[c_idx] = siloed_commitments[c_idx];              // create random read requests
        hints[c_idx] = fr(c_idx);                                      // ^ will match each other!
        read_request_membership_witnesses[c_idx].is_transient = true;  // ordering circuit only allows transient reads
    }
    read_requests[3] = NT::fr::random_element();  // force one read request not to match

    auto& previous_kernel = private_inputs_inner.previous_kernel;

    previous_kernel.public_inputs.end.new_commitments = siloed_commitments;
    previous_kernel.public_inputs.end.read_requests = read_requests;

    PrivateKernelInputsOrdering<NT> private_inputs{ previous_kernel, hints };

    DummyBuilder builder = DummyBuilder("native_private_kernel_ordering_tests__native_read_request_unknown_fails");
    native_private_kernel_circuit_ordering(builder, private_inputs);

    auto failure = builder.get_first_failure();
    ASSERT_EQ(failure.code, CircuitErrorCode::PRIVATE_KERNEL__TRANSIENT_READ_REQUEST_NO_MATCH);
}

TEST_F(native_private_kernel_ordering_tests, native_squash_one_of_one_transient_matches_works)
{
    auto private_inputs_inner = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    std::array<fr, MAX_NEW_COMMITMENTS_PER_TX> new_commitments{};
    std::array<fr, MAX_NEW_NULLIFIERS_PER_TX> new_nullifiers{};
    std::array<fr, MAX_NEW_NULLIFIERS_PER_TX> nullifier_commitments{};

    const auto commitment0 = fr(213);
    new_commitments[0] = commitment0;

    new_nullifiers[0] = fr(32);
    nullifier_commitments[0] = commitment0;

    auto& previous_kernel = private_inputs_inner.previous_kernel;

    previous_kernel.public_inputs.end.new_commitments = new_commitments;
    previous_kernel.public_inputs.end.new_nullifiers = new_nullifiers;
    previous_kernel.public_inputs.end.nullified_commitments = nullifier_commitments;

    // Correct nullifier_commitment hint for new_nullifiers[0] == 0 is correct due to the default
    // initialization of the array.
    PrivateKernelInputsOrdering<NT> private_inputs{ .previous_kernel = previous_kernel };

    DummyBuilder builder =
        DummyBuilder("native_private_kernel_ordering_tests__native_squash_one_of_one_transient_matches_works");
    auto public_inputs = native_private_kernel_circuit_ordering(builder, private_inputs);

    ASSERT_FALSE(builder.failed()) << "failure: " << builder.get_first_failure();
    ASSERT_TRUE(array_length(public_inputs.end.new_commitments) == 0);  // 1/1 squashed
    ASSERT_TRUE(array_length(public_inputs.end.new_nullifiers) == 0);   // 1/1 squashed
}

TEST_F(native_private_kernel_ordering_tests, native_squash_one_of_two_transient_matches_works)
{
    auto private_inputs_inner = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    std::array<fr, MAX_NEW_COMMITMENTS_PER_TX> new_commitments{};
    std::array<fr, MAX_NEW_NULLIFIERS_PER_TX> new_nullifiers{};
    std::array<fr, MAX_NEW_NULLIFIERS_PER_TX> nullifier_commitments{};

    const auto commitment1 = fr(213);
    new_commitments[0] = fr(763);
    new_commitments[1] = commitment1;

    new_nullifiers[0] = fr(32);
    nullifier_commitments[0] = commitment1;

    auto& previous_kernel = private_inputs_inner.previous_kernel;

    previous_kernel.public_inputs.end.new_commitments = new_commitments;
    previous_kernel.public_inputs.end.new_nullifiers = new_nullifiers;
    previous_kernel.public_inputs.end.nullified_commitments = nullifier_commitments;

    PrivateKernelInputsOrdering<NT> private_inputs{ .previous_kernel = previous_kernel,
                                                    .nullifier_commitment_hints =
                                                        std::array<fr, MAX_NEW_NULLIFIERS_PER_TX>{ 1 } };

    DummyBuilder builder =
        DummyBuilder("native_private_kernel_ordering_tests__native_squash_one_of_two_transient_matches_works");
    auto public_inputs = native_private_kernel_circuit_ordering(builder, private_inputs);

    ASSERT_FALSE(builder.failed()) << "failure: " << builder.get_first_failure();
    ASSERT_TRUE(array_length(public_inputs.end.new_commitments) == 1);  // 1/2 squashed
    ASSERT_TRUE(array_length(public_inputs.end.new_nullifiers) == 0);   // 1/1 squashed
}

TEST_F(native_private_kernel_ordering_tests, native_squash_two_of_two_transient_matches_works)
{
    auto private_inputs_inner = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    std::array<fr, MAX_NEW_COMMITMENTS_PER_TX> new_commitments{};
    std::array<fr, MAX_NEW_NULLIFIERS_PER_TX> new_nullifiers{};
    std::array<fr, MAX_NEW_NULLIFIERS_PER_TX> nullifier_commitments{};

    const auto commitment0 = fr(763);
    const auto commitment1 = fr(213);
    new_commitments[0] = commitment0;
    new_commitments[1] = commitment1;

    new_nullifiers[0] = fr(32);
    new_nullifiers[1] = fr(43);
    nullifier_commitments[0] = commitment1;
    nullifier_commitments[1] = commitment0;

    auto& previous_kernel = private_inputs_inner.previous_kernel;

    previous_kernel.public_inputs.end.new_commitments = new_commitments;
    previous_kernel.public_inputs.end.new_nullifiers = new_nullifiers;
    previous_kernel.public_inputs.end.nullified_commitments = nullifier_commitments;

    PrivateKernelInputsOrdering<NT> private_inputs{ .previous_kernel = previous_kernel,
                                                    .nullifier_commitment_hints =
                                                        std::array<fr, MAX_NEW_NULLIFIERS_PER_TX>{ 1 } };

    DummyBuilder builder =
        DummyBuilder("native_private_kernel_ordering_tests__native_squash_two_of_two_transient_matches_works");
    auto public_inputs = native_private_kernel_circuit_ordering(builder, private_inputs);

    ASSERT_FALSE(builder.failed()) << "failure: " << builder.get_first_failure();
    ASSERT_TRUE(array_length(public_inputs.end.new_commitments) == 0);  // 2/2 squashed
    ASSERT_TRUE(array_length(public_inputs.end.new_nullifiers) == 0);   // 2/2 squashed
}

TEST_F(native_private_kernel_ordering_tests, native_empty_nullified_commitment_means_persistent_nullifier_0)
{
    auto private_inputs_inner = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    std::array<fr, MAX_NEW_COMMITMENTS_PER_TX> new_commitments{};
    std::array<fr, MAX_NEW_NULLIFIERS_PER_TX> new_nullifiers{};
    std::array<fr, MAX_NEW_NULLIFIERS_PER_TX> nullifier_commitments{};

    new_commitments[0] = fr(213);

    new_nullifiers[0] = fr(32);
    nullifier_commitments[0] = fr(EMPTY_NULLIFIED_COMMITMENT);

    auto& previous_kernel = private_inputs_inner.previous_kernel;

    previous_kernel.public_inputs.end.new_commitments = new_commitments;
    previous_kernel.public_inputs.end.new_nullifiers = new_nullifiers;
    previous_kernel.public_inputs.end.nullified_commitments = nullifier_commitments;

    PrivateKernelInputsOrdering<NT> private_inputs{ .previous_kernel = previous_kernel };

    DummyBuilder builder = DummyBuilder(
        "native_private_kernel_ordering_tests__native_empty_nullified_commitment_means_persistent_nullifier_0");
    auto public_inputs = native_private_kernel_circuit_ordering(builder, private_inputs);

    ASSERT_FALSE(builder.failed()) << "failure: " << builder.get_first_failure();
    // nullifier and commitment present at output (will become persistant)
    ASSERT_TRUE(array_length(public_inputs.end.new_commitments) == 1);
    ASSERT_TRUE(array_length(public_inputs.end.new_nullifiers) == 1);
}

// same as previous test, but this time there are 0 commitments!
TEST_F(native_private_kernel_ordering_tests, native_empty_nullified_commitment_means_persistent_nullifier_1)
{
    auto private_inputs_inner = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    std::array<fr, MAX_NEW_COMMITMENTS_PER_TX> new_commitments{};
    std::array<fr, MAX_NEW_NULLIFIERS_PER_TX> new_nullifiers{};
    std::array<fr, MAX_NEW_NULLIFIERS_PER_TX> nullifier_commitments{};

    new_nullifiers[0] = fr(32);
    nullifier_commitments[0] = fr(EMPTY_NULLIFIED_COMMITMENT);

    auto& previous_kernel = private_inputs_inner.previous_kernel;

    previous_kernel.public_inputs.end.new_commitments = new_commitments;
    previous_kernel.public_inputs.end.new_nullifiers = new_nullifiers;
    previous_kernel.public_inputs.end.nullified_commitments = nullifier_commitments;

    PrivateKernelInputsOrdering<NT> private_inputs{ .previous_kernel = previous_kernel };

    DummyBuilder builder = DummyBuilder(
        "native_private_kernel_ordering_tests__native_empty_nullified_commitment_means_persistent_nullifier_1");
    auto public_inputs = native_private_kernel_circuit_ordering(builder, private_inputs);

    ASSERT_FALSE(builder.failed()) << "failure: " << builder.get_first_failure();
    ASSERT_TRUE(array_length(public_inputs.end.new_commitments) == 0);
    // nullifier present at output (will become persistant)
    ASSERT_TRUE(array_length(public_inputs.end.new_nullifiers) == 1);
}

TEST_F(native_private_kernel_ordering_tests, 0th_nullifier_zero_fails)
{
    auto private_inputs_inner = do_private_call_get_kernel_inputs_inner(false, deposit, standard_test_args());

    std::array<fr, MAX_NEW_NULLIFIERS_PER_TX> new_nullifiers{};
    auto& previous_kernel = private_inputs_inner.previous_kernel;
    previous_kernel.public_inputs.end.new_nullifiers = new_nullifiers;
    PrivateKernelInputsOrdering<NT> private_inputs{ .previous_kernel = previous_kernel };

    DummyBuilder builder = DummyBuilder("native_private_kernel_ordering_tests__0th_nullifier_zero_fails");
    auto public_inputs = native_private_kernel_circuit_ordering(builder, private_inputs);

    ASSERT_TRUE(builder.failed());
    auto failure = builder.get_first_failure();
    ASSERT_EQ(failure.code, CircuitErrorCode::PRIVATE_KERNEL__0TH_NULLLIFIER_IS_ZERO);
}

}  // namespace aztec3::circuits::kernel::private_kernel
