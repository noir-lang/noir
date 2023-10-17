#include "sumcheck_round.hpp"
#include "barretenberg/honk/flavor/ultra.hpp"

#include <gtest/gtest.h>

using namespace proof_system::honk;
using namespace proof_system::honk::sumcheck;

using barretenberg::BarycentricData;
using barretenberg::PowUnivariate;
using barretenberg::Univariate;

using Flavor = flavor::Ultra;
using FF = typename Flavor::FF;

namespace test_sumcheck_round {

/**
 * @brief Test SumcheckRound functions for operations on tuples (and tuples of tuples) of Univariates
 *
 */
TEST(SumcheckRound, TupleOfTuplesOfUnivariates)
{
    using Flavor = proof_system::honk::flavor::Ultra;
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
    using Flavor = proof_system::honk::flavor::Ultra;
    using Utils = barretenberg::RelationUtils<Flavor>;
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
    Utils::scale_and_batch_elements(tuple_of_arrays, challenge, running_challenge, result);

    // Repeat the batching process manually
    auto result_expected =
        evaluations_1[0] * 1 + evaluations_2[0] * challenge + evaluations_2[1] * challenge * challenge;

    // Compare batched result
    EXPECT_EQ(result, result_expected);

    // Reinitialize univariate accumulators to zero
    Utils::zero_elements(tuple_of_arrays);

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
    using Flavor = proof_system::honk::flavor::Ultra;
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

    // Construct two tuples of tuples of univariates
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
