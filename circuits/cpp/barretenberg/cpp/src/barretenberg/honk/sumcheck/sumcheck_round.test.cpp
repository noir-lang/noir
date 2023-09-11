#include "sumcheck_round.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/honk/flavor/standard.hpp"
#include "barretenberg/numeric/random/engine.hpp"
#include "barretenberg/polynomials/univariate.hpp"

#include <tuple>

#include "barretenberg/common/mem.hpp"
#include <gtest/gtest.h>
/**
 * We want to test if the univariate (S_l in the thesis) computed by the prover in a particular round is correct. We
 * also want to verify given the purported evaluations of all the relevant polynomials, the verifer can correctly verify
 * the purported evaluation of S_l. For the prover, we use a couple of methods to compute the univariate by the sumcheck
 * method `compute_univariate` and by step by step manual computation respectively. For the verifier, we follow a
 * similar approach.
 */

using namespace proof_system::honk;
using namespace proof_system::honk::sumcheck;
using namespace proof_system;

using barretenberg::BarycentricData;
using barretenberg::PowUnivariate;
using barretenberg::Univariate;

using Flavor = flavor::Standard;
using FF = typename Flavor::FF;
using ProverPolynomials = typename Flavor::ProverPolynomials;
using ClaimedEvaluations = typename Flavor::ClaimedEvaluations;

const size_t NUM_POLYNOMIALS = Flavor::NUM_ALL_ENTITIES;
const size_t max_relation_length = Flavor::MAX_RANDOM_RELATION_LENGTH;
const size_t input_polynomial_length = 2;

namespace test_sumcheck_round {

// The below two methods are used in the test ComputeUnivariateProver
static Univariate<FF, max_relation_length> compute_round_univariate(
    std::array<std::array<FF, input_polynomial_length>, NUM_POLYNOMIALS>& input_polynomials,
    const RelationParameters<FF>& relation_parameters,
    const FF alpha)
{
    size_t round_size = 2;
    // Improvement(Cody): This is ugly? Maye supply some/all of this data through "flavor" class?
    auto round = SumcheckProverRound<Flavor>(round_size);

    ProverPolynomials full_polynomials;
    full_polynomials.w_l = input_polynomials[0];
    full_polynomials.w_r = input_polynomials[1];
    full_polynomials.w_o = input_polynomials[2];
    full_polynomials.z_perm = input_polynomials[3];
    full_polynomials.z_perm_shift = input_polynomials[4];
    full_polynomials.q_m = input_polynomials[5];
    full_polynomials.q_l = input_polynomials[6];
    full_polynomials.q_r = input_polynomials[7];
    full_polynomials.q_o = input_polynomials[8];
    full_polynomials.q_c = input_polynomials[9];
    full_polynomials.sigma_1 = input_polynomials[10];
    full_polynomials.sigma_2 = input_polynomials[11];
    full_polynomials.sigma_3 = input_polynomials[12];
    full_polynomials.id_1 = input_polynomials[13];
    full_polynomials.id_2 = input_polynomials[14];
    full_polynomials.id_3 = input_polynomials[15];
    full_polynomials.lagrange_first = input_polynomials[16];
    full_polynomials.lagrange_last = input_polynomials[17];

    PowUnivariate<FF> pow_zeta(1);
    Univariate<FF, max_relation_length> round_univariate =
        round.compute_univariate(full_polynomials, relation_parameters, pow_zeta, alpha);
    return round_univariate;
}

static Univariate<FF, max_relation_length> compute_expected_round_univariate(
    std::array<Univariate<FF, input_polynomial_length>, NUM_POLYNOMIALS>& input_univariates,
    const RelationParameters<FF>& relation_parameters,
    const FF alpha)
{
    BarycentricData<FF, input_polynomial_length, max_relation_length> barycentric_2_to_max =
        BarycentricData<FF, input_polynomial_length, max_relation_length>();
    std::array<Univariate<FF, max_relation_length>, NUM_POLYNOMIALS> extended_univariates;
    for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
        extended_univariates[i] = barycentric_2_to_max.extend(input_univariates[i]);
    }
    auto w_l_univariate = Univariate<FF, max_relation_length>(extended_univariates[0]);
    auto w_r_univariate = Univariate<FF, max_relation_length>(extended_univariates[1]);
    auto w_o_univariate = Univariate<FF, max_relation_length>(extended_univariates[2]);
    auto z_perm_univariate = Univariate<FF, max_relation_length>(extended_univariates[3]);
    auto z_perm_shift_univariate =
        Univariate<FF, max_relation_length>(extended_univariates[4]); // this is not real shifted data
    auto q_m_univariate = Univariate<FF, max_relation_length>(extended_univariates[5]);
    auto q_l_univariate = Univariate<FF, max_relation_length>(extended_univariates[6]);
    auto q_r_univariate = Univariate<FF, max_relation_length>(extended_univariates[7]);
    auto q_o_univariate = Univariate<FF, max_relation_length>(extended_univariates[8]);
    auto q_c_univariate = Univariate<FF, max_relation_length>(extended_univariates[9]);
    auto sigma_1_univariate = Univariate<FF, max_relation_length>(extended_univariates[10]);
    auto sigma_2_univariate = Univariate<FF, max_relation_length>(extended_univariates[11]);
    auto sigma_3_univariate = Univariate<FF, max_relation_length>(extended_univariates[12]);
    auto id_1_univariate = Univariate<FF, max_relation_length>(extended_univariates[13]);
    auto id_2_univariate = Univariate<FF, max_relation_length>(extended_univariates[14]);
    auto id_3_univariate = Univariate<FF, max_relation_length>(extended_univariates[15]);
    auto lagrange_first_univariate = Univariate<FF, max_relation_length>(extended_univariates[16]);
    auto lagrange_last_univariate = Univariate<FF, max_relation_length>(extended_univariates[17]);

    auto expected_arithmetic_relation =
        ((q_m_univariate * w_r_univariate * w_l_univariate) + (q_r_univariate * w_r_univariate) +
         (q_l_univariate * w_l_univariate) + (q_o_univariate * w_o_univariate) + (q_c_univariate));
    auto expected_grand_product_computation_relation =
        ((z_perm_univariate + lagrange_first_univariate) *
         (w_l_univariate + id_1_univariate * relation_parameters.beta + relation_parameters.gamma) *
         (w_r_univariate + id_2_univariate * relation_parameters.beta + relation_parameters.gamma) *
         (w_o_univariate + id_3_univariate * relation_parameters.beta + relation_parameters.gamma));
    expected_grand_product_computation_relation -=
        ((z_perm_shift_univariate + lagrange_last_univariate * relation_parameters.public_input_delta) *
         (w_l_univariate + sigma_1_univariate * relation_parameters.beta + relation_parameters.gamma) *
         (w_r_univariate + sigma_2_univariate * relation_parameters.beta + relation_parameters.gamma) *
         (w_o_univariate + sigma_3_univariate * relation_parameters.beta + relation_parameters.gamma));
    auto expected_grand_product_initialization_relation = (z_perm_shift_univariate * lagrange_last_univariate);
    Univariate<FF, max_relation_length> expected_round_univariate =
        expected_arithmetic_relation + expected_grand_product_computation_relation * alpha +
        expected_grand_product_initialization_relation * alpha.sqr();
    return expected_round_univariate;
}

// The below two methods are used in the test ComputeUnivariateVerifier
static FF compute_full_purported_value(std::array<FF, NUM_POLYNOMIALS>& input_values,
                                       const RelationParameters<FF>& relation_parameters,
                                       const FF alpha)
{
    ClaimedEvaluations purported_evaluations;
    purported_evaluations.w_l = input_values[0];
    purported_evaluations.w_r = input_values[1];
    purported_evaluations.w_o = input_values[2];
    purported_evaluations.z_perm = input_values[3];
    purported_evaluations.z_perm_shift = input_values[4];
    purported_evaluations.q_m = input_values[5];
    purported_evaluations.q_l = input_values[6];
    purported_evaluations.q_r = input_values[7];
    purported_evaluations.q_o = input_values[8];
    purported_evaluations.q_c = input_values[9];
    purported_evaluations.sigma_1 = input_values[10];
    purported_evaluations.sigma_2 = input_values[11];
    purported_evaluations.sigma_3 = input_values[12];
    purported_evaluations.id_1 = input_values[13];
    purported_evaluations.id_2 = input_values[14];
    purported_evaluations.id_3 = input_values[15];
    purported_evaluations.lagrange_first = input_values[16];
    purported_evaluations.lagrange_last = input_values[17];

    auto round = SumcheckVerifierRound<Flavor>();
    PowUnivariate<FF> pow_univariate(1);
    FF full_purported_value = round.compute_full_honk_relation_purported_value(
        purported_evaluations, relation_parameters, pow_univariate, alpha);
    return full_purported_value;
}

static FF compute_full_purported_value_expected(std::array<FF, NUM_POLYNOMIALS>& input_values,
                                                const RelationParameters<FF>& relation_parameters,
                                                const FF alpha)
{
    FF w_l = input_values[0];
    FF w_r = input_values[1];
    FF w_o = input_values[2];
    FF z_perm = input_values[3];
    FF z_perm_shift = input_values[4];
    FF q_m = input_values[5];
    FF q_l = input_values[6];
    FF q_r = input_values[7];
    FF q_o = input_values[8];
    FF q_c = input_values[9];
    FF sigma_1 = input_values[10];
    FF sigma_2 = input_values[11];
    FF sigma_3 = input_values[12];
    FF id_1 = input_values[13];
    FF id_2 = input_values[14];
    FF id_3 = input_values[15];
    FF lagrange_first = input_values[16];
    FF lagrange_last = input_values[17];
    auto expected_arithmetic_relation = (q_m * w_r * w_l) + (q_r * w_r) + (q_l * w_l) + (q_o * w_o) + q_c;
    auto expected_grand_product_computation_relation =
        (z_perm + lagrange_first) * (w_l + id_1 * relation_parameters.beta + relation_parameters.gamma) *
        (w_r + id_2 * relation_parameters.beta + relation_parameters.gamma) *
        (w_o + id_3 * relation_parameters.beta + relation_parameters.gamma);
    expected_grand_product_computation_relation -=
        (z_perm_shift + lagrange_last * relation_parameters.public_input_delta) *
        (w_l + sigma_1 * relation_parameters.beta + relation_parameters.gamma) *
        (w_r + sigma_2 * relation_parameters.beta + relation_parameters.gamma) *
        (w_o + sigma_3 * relation_parameters.beta + relation_parameters.gamma);
    auto expected_grand_product_initialization_relation = z_perm_shift * lagrange_last;
    auto expected_full_purported_value = expected_arithmetic_relation +
                                         expected_grand_product_computation_relation * alpha +
                                         expected_grand_product_initialization_relation * alpha.sqr();
    return expected_full_purported_value;
}

TEST(SumcheckRound, ComputeUnivariateProver)
{
    auto run_test = [](bool is_random_input) {
        if (is_random_input) {
            std::array<std::array<FF, input_polynomial_length>, NUM_POLYNOMIALS> input_polynomials;
            for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
                input_polynomials[i] = { FF::random_element(), FF::random_element() };
            }

            const FF alpha = FF::random_element();
            const RelationParameters<FF> relation_parameters = RelationParameters<FF>{
                .beta = FF::random_element(), .gamma = FF::random_element(), .public_input_delta = FF::random_element()
            };

            auto round_univariate = compute_round_univariate(input_polynomials, relation_parameters, alpha);

            // Compute round_univariate manually
            std::array<Univariate<FF, input_polynomial_length>, NUM_POLYNOMIALS> input_univariates;
            for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
                input_univariates[i] = Univariate<FF, input_polynomial_length>(input_polynomials[i]);
            }
            auto expected_round_univariate =
                compute_expected_round_univariate(input_univariates, relation_parameters, alpha);
            EXPECT_EQ(round_univariate, expected_round_univariate);
        } else {
            std::array<std::array<FF, input_polynomial_length>, NUM_POLYNOMIALS> input_polynomials;
            for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
                input_polynomials[i] = { 1, 2 };
            }
            const FF alpha = 1;
            const RelationParameters<FF> relation_parameters =
                RelationParameters<FF>{ .beta = 1, .gamma = 1, .public_input_delta = 1 };
            auto round_univariate = compute_round_univariate(input_polynomials, relation_parameters, alpha);
            // Compute round_univariate manually
            std::array<Univariate<FF, input_polynomial_length>, NUM_POLYNOMIALS> input_univariates;
            for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
                input_univariates[i] = Univariate<FF, input_polynomial_length>(input_polynomials[i]);
            }
            // expected_round_univariate = { 6, 26, 66, 132, 230, 366 }
            auto expected_round_univariate =
                compute_expected_round_univariate(input_univariates, relation_parameters, alpha);
            EXPECT_EQ(round_univariate, expected_round_univariate);
        };
    };
    run_test(/* is_random_input=*/false);
    run_test(/* is_random_input=*/true);
}

TEST(SumcheckRound, ComputeUnivariateVerifier)
{
    auto run_test = [](bool is_random_input) {
        if (is_random_input) {
            std::array<FF, NUM_POLYNOMIALS> input_values;
            for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
                input_values[i] = FF::random_element();
            }
            const FF alpha = FF::random_element();
            const RelationParameters<FF> relation_parameters = RelationParameters<FF>{
                .beta = FF::random_element(), .gamma = FF::random_element(), .public_input_delta = FF::random_element()
            };
            auto full_purported_value = compute_full_purported_value(input_values, relation_parameters, alpha);
            // Compute round_univariate manually
            auto expected_full_purported_value =
                compute_full_purported_value_expected(input_values, relation_parameters, alpha);
            EXPECT_EQ(full_purported_value, expected_full_purported_value);
        } else {
            std::array<FF, NUM_POLYNOMIALS> input_values;
            for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
                input_values[i] = FF(2);
            }
            const FF alpha = 1;
            const RelationParameters<FF> relation_parameters =
                RelationParameters<FF>{ .beta = 1, .gamma = 1, .public_input_delta = 1 };
            auto full_purported_value = compute_full_purported_value(input_values, relation_parameters, alpha);
            // Compute round_univariate manually
            auto expected_full_purported_value =
                compute_full_purported_value_expected(input_values, relation_parameters, alpha);
            EXPECT_EQ(full_purported_value, expected_full_purported_value);
        };
    };
    run_test(/* is_random_input=*/false);
    run_test(/* is_random_input=*/true);
}

/**
 * @brief Test utility functions for applying operations to tuple of tuple of Univariates
 *
 */
TEST(SumcheckRound, TupleOfTuplesOfUnivariates)
{
    using Flavor = proof_system::honk::flavor::Standard;
    using FF = typename Flavor::FF;

    // Define three linear univariates of different sizes
    Univariate<FF, 3> univariate_1({ 1, 2, 3 });
    Univariate<FF, 2> univariate_2({ 2, 4 });
    Univariate<FF, 5> univariate_3({ 3, 4, 5, 6, 7 });
    const size_t MAX_LENGTH = 5;

    // Instantiate some barycentric extension utility classes
    auto barycentric_util_1 = BarycentricData<FF, 3, MAX_LENGTH>();
    auto barycentric_util_2 = BarycentricData<FF, 2, MAX_LENGTH>();
    auto barycentric_util_3 = BarycentricData<FF, 5, MAX_LENGTH>();

    // Construct a tuple of tuples of the form { {univariate_1}, {univariate_2, univariate_3} }
    auto tuple_of_tuples = std::make_tuple(std::make_tuple(univariate_1), std::make_tuple(univariate_2, univariate_3));

    // Use scale_univariate_accumulators to scale by challenge powers
    FF challenge = 5;
    FF running_challenge = 1;
    SumcheckProverRound<Flavor>::scale_univariates(tuple_of_tuples, challenge, running_challenge);

    // Use extend_and_batch_univariates to extend to MAX_LENGTH then accumulate
    PowUnivariate<FF> pow_univariate(1);
    auto result = Univariate<FF, MAX_LENGTH>();
    SumcheckProverRound<Flavor>::extend_and_batch_univariates(tuple_of_tuples, pow_univariate, result);

    // Repeat the batching process manually
    auto result_expected = barycentric_util_1.extend(univariate_1) * 1 +
                           barycentric_util_2.extend(univariate_2) * challenge +
                           barycentric_util_3.extend(univariate_3) * challenge * challenge;

    // Compare final batched univarites
    EXPECT_EQ(result, result_expected);

    // Reinitialize univariate accumulators to zero
    SumcheckProverRound<Flavor>::zero_univariates(tuple_of_tuples);

    // Check that reinitialization was successful
    Univariate<FF, 3> expected_1({ 0, 0, 0 });
    Univariate<FF, 2> expected_2({ 0, 0 });
    Univariate<FF, 5> expected_3({ 0, 0, 0, 0, 0 });
    EXPECT_EQ(std::get<0>(std::get<0>(tuple_of_tuples)), expected_1);
    EXPECT_EQ(std::get<0>(std::get<1>(tuple_of_tuples)), expected_2);
    EXPECT_EQ(std::get<1>(std::get<1>(tuple_of_tuples)), expected_3);
}

/**
 * @brief Test utility functions for applying operations to tuple of std::arrays of field elements
 *
 */
TEST(SumcheckRound, TuplesOfEvaluationArrays)
{
    using Flavor = proof_system::honk::flavor::Standard;
    using FF = typename Flavor::FF;

    // Define two arrays of arbitrary elements
    std::array<FF, 1> evaluations_1 = { 4 };
    std::array<FF, 2> evaluations_2 = { 6, 2 };

    // Construct a tuple
    auto tuple_of_arrays = std::make_tuple(evaluations_1, evaluations_2);

    // Use scale_and_batch_elements to scale by challenge powers
    FF challenge = 5;
    FF running_challenge = 1;
    FF result = 0;
    SumcheckVerifierRound<Flavor>::scale_and_batch_elements(tuple_of_arrays, challenge, running_challenge, result);

    // Repeat the batching process manually
    auto result_expected =
        evaluations_1[0] * 1 + evaluations_2[0] * challenge + evaluations_2[1] * challenge * challenge;

    // Compare batched result
    EXPECT_EQ(result, result_expected);

    // Reinitialize univariate accumulators to zero
    SumcheckVerifierRound<Flavor>::zero_elements(tuple_of_arrays);

    EXPECT_EQ(std::get<0>(tuple_of_arrays)[0], 0);
    EXPECT_EQ(std::get<1>(tuple_of_arrays)[0], 0);
    EXPECT_EQ(std::get<1>(tuple_of_arrays)[1], 0);
}

/**
 * @brief Test utility functions for adding two tuples of tuples of Univariates
 *
 */
TEST(SumcheckRound, AddTuplesOfTuplesOfUnivariates)
{
    using Flavor = proof_system::honk::flavor::Standard;
    using FF = typename Flavor::FF;

    // Define some arbitrary univariates
    Univariate<FF, 2> univariate_1({ 1, 2 });
    Univariate<FF, 2> univariate_2({ 2, 4 });
    Univariate<FF, 3> univariate_3({ 3, 4, 5 });

    Univariate<FF, 2> univariate_4({ 3, 6 });
    Univariate<FF, 2> univariate_5({ 8, 1 });
    Univariate<FF, 3> univariate_6({ 3, 7, 1 });

    Univariate<FF, 2> expected_sum_1 = univariate_1 + univariate_4;
    Univariate<FF, 2> expected_sum_2 = univariate_2 + univariate_5;
    Univariate<FF, 3> expected_sum_3 = univariate_3 + univariate_6;

    // Construct two tuples of tuples
    auto tuple_of_tuples_1 =
        std::make_tuple(std::make_tuple(univariate_1), std::make_tuple(univariate_2, univariate_3));
    auto tuple_of_tuples_2 =
        std::make_tuple(std::make_tuple(univariate_4), std::make_tuple(univariate_5, univariate_6));

    SumcheckProverRound<Flavor>::add_nested_tuples(tuple_of_tuples_1, tuple_of_tuples_2);

    EXPECT_EQ(std::get<0>(std::get<0>(tuple_of_tuples_1)), expected_sum_1);
    EXPECT_EQ(std::get<0>(std::get<1>(tuple_of_tuples_1)), expected_sum_2);
    EXPECT_EQ(std::get<1>(std::get<1>(tuple_of_tuples_1)), expected_sum_3);
}

} // namespace test_sumcheck_round
