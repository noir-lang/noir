#include "barretenberg/honk/sumcheck/relations/lookup_grand_product_relation.hpp"
#include "barretenberg/honk/sumcheck/relations/ultra_arithmetic_relation.hpp"
#include "barretenberg/honk/sumcheck/relations/ultra_arithmetic_relation_secondary.hpp"
#include "relation.hpp"
#include "barretenberg/honk/flavor/flavor.hpp"
#include "arithmetic_relation.hpp"
#include "grand_product_initialization_relation.hpp"
#include "grand_product_computation_relation.hpp"
#include "../polynomials/univariate.hpp"
#include "../polynomials/barycentric_data.hpp"

#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/numeric/random/engine.hpp"

#include <cstddef>
#include <gtest/gtest.h>
using namespace proof_system::honk::sumcheck;
/**
 * We want to test if all three relations (namely, ArithmeticRelation, GrandProductComputationRelation,
 * GrandProductInitializationRelation) provide correct contributions by manually computing their
 * contributions with deterministic and random inputs. The relations are supposed to work with
 * univariates (edges) of degree one (length 2) and spit out polynomials of corresponding degrees. We have
 * MAX_RELATION_LENGTH = 5, meaning the output of a relation can atmost be a degree 5 polynomial. Hence,
 * we use a method compute_mock_extended_edges() which starts with degree one input polynomial (two evaluation
 points),
 * extends them (using barycentric formula) to six evaluation points, and stores them to an array of polynomials.
 */
static const size_t INPUT_UNIVARIATE_LENGTH = 2;

namespace proof_system::honk_relation_tests {

template <class FF> class RelationConsistency : public testing::Test {
  public:
    template <size_t t> using Univariate = Univariate<FF, t>;
    template <size_t t> using UnivariateView = UnivariateView<FF, t>;
    using POLYNOMIAL = proof_system::honk::StandardArithmetization::POLYNOMIAL;
    // TODO(#225)(Adrian): Accept FULL_RELATION_LENGTH as a template parameter for this function only, so that the test
    // can decide to which degree the polynomials must be extended. Possible accept an existing list of "edges" and
    // extend them to the degree.
    template <size_t FULL_RELATION_LENGTH, size_t NUM_POLYNOMIALS>
    static std::array<Univariate<FULL_RELATION_LENGTH>, NUM_POLYNOMIALS> compute_mock_extended_edges(
        std::array<Univariate<INPUT_UNIVARIATE_LENGTH>, NUM_POLYNOMIALS>& input_univariates)
    {
        BarycentricData<FF, INPUT_UNIVARIATE_LENGTH, FULL_RELATION_LENGTH> barycentric_2_to_max =
            BarycentricData<FF, INPUT_UNIVARIATE_LENGTH, FULL_RELATION_LENGTH>();
        std::array<Univariate<FULL_RELATION_LENGTH>, NUM_POLYNOMIALS> extended_univariates;
        for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
            extended_univariates[i] = barycentric_2_to_max.extend(input_univariates[i]);
        }
        auto w_l = Univariate<FULL_RELATION_LENGTH>(extended_univariates[0]);
        auto w_r = Univariate<FULL_RELATION_LENGTH>(extended_univariates[1]);
        auto w_o = Univariate<FULL_RELATION_LENGTH>(extended_univariates[2]);
        auto z_perm = Univariate<FULL_RELATION_LENGTH>(extended_univariates[3]);
        auto z_perm_shift = Univariate<FULL_RELATION_LENGTH>(extended_univariates[4]); // this is not real shifted data
        auto q_m = Univariate<FULL_RELATION_LENGTH>(extended_univariates[5]);
        auto q_l = Univariate<FULL_RELATION_LENGTH>(extended_univariates[6]);
        auto q_r = Univariate<FULL_RELATION_LENGTH>(extended_univariates[7]);
        auto q_o = Univariate<FULL_RELATION_LENGTH>(extended_univariates[8]);
        auto q_c = Univariate<FULL_RELATION_LENGTH>(extended_univariates[9]);
        auto sigma_1 = Univariate<FULL_RELATION_LENGTH>(extended_univariates[10]);
        auto sigma_2 = Univariate<FULL_RELATION_LENGTH>(extended_univariates[11]);
        auto sigma_3 = Univariate<FULL_RELATION_LENGTH>(extended_univariates[12]);
        auto id_1 = Univariate<FULL_RELATION_LENGTH>(extended_univariates[13]);
        auto id_2 = Univariate<FULL_RELATION_LENGTH>(extended_univariates[14]);
        auto id_3 = Univariate<FULL_RELATION_LENGTH>(extended_univariates[15]);
        auto lagrange_first = Univariate<FULL_RELATION_LENGTH>(extended_univariates[16]);
        auto lagrange_last = Univariate<FULL_RELATION_LENGTH>(extended_univariates[17]);
        // Construct extended edges array in order determined by enum
        std::array<Univariate<FULL_RELATION_LENGTH>, NUM_POLYNOMIALS> extended_edges;
        extended_edges[POLYNOMIAL::W_L] = w_l;
        extended_edges[POLYNOMIAL::W_R] = w_r;
        extended_edges[POLYNOMIAL::W_O] = w_o;
        extended_edges[POLYNOMIAL::Z_PERM] = z_perm;
        extended_edges[POLYNOMIAL::Z_PERM_SHIFT] = z_perm_shift;
        extended_edges[POLYNOMIAL::Q_M] = q_m;
        extended_edges[POLYNOMIAL::Q_L] = q_l;
        extended_edges[POLYNOMIAL::Q_R] = q_r;
        extended_edges[POLYNOMIAL::Q_O] = q_o;
        extended_edges[POLYNOMIAL::Q_C] = q_c;
        extended_edges[POLYNOMIAL::SIGMA_1] = sigma_1;
        extended_edges[POLYNOMIAL::SIGMA_2] = sigma_2;
        extended_edges[POLYNOMIAL::SIGMA_3] = sigma_3;
        extended_edges[POLYNOMIAL::ID_1] = id_1;
        extended_edges[POLYNOMIAL::ID_2] = id_2;
        extended_edges[POLYNOMIAL::ID_3] = id_3;
        extended_edges[POLYNOMIAL::LAGRANGE_FIRST] = lagrange_first;
        extended_edges[POLYNOMIAL::LAGRANGE_LAST] = lagrange_last;

        return extended_edges;
    }

    /**
     * @brief Returns randomly sampled parameters to feed to the relations.
     *
     * @return RelationParameters<FF>
     */
    RelationParameters<FF> compute_mock_relation_parameters()
    {
        return { .eta = FF::random_element(),
                 .beta = FF::random_element(),
                 .gamma = FF::random_element(),
                 .public_input_delta = FF::random_element(),
                 .lookup_grand_product_delta = FF::random_element() };
    }

    /**
     * @brief Given an array of Univariates, create a new array containing only the i-th evaluations
     * of all the univariates.
     *
     * @note Not really optimized, mainly used for testing that the relations evaluate to the same value when
     * evaluated as Univariates, Expressions, or index-by-index
     * @todo(Adrian) Maybe this is more helpful as part of a `check_logic` function.
     *
     * @tparam NUM_UNIVARIATES number of univariates in the input array (deduced from `univariates`)
     * @tparam univariate_length number of evaluations (deduced from `univariates`)
     * @param univariates array of Univariates
     * @param i index of the evaluations we want to take from each univariate
     * @return std::array<FF, NUM_UNIVARIATES> such that result[j] = univariates[j].value_at(i)
     */
    template <std::size_t NUM_UNIVARIATES, size_t univariate_length>
    static std::array<FF, NUM_UNIVARIATES> transposed_univariate_array_at(
        const std::array<Univariate<univariate_length>, NUM_UNIVARIATES>& univariates, size_t i)
    {
        ASSERT(i < univariate_length);
        std::array<FF, NUM_UNIVARIATES> result;
        for (size_t j = 0; j < NUM_UNIVARIATES; ++j) {
            result[j] = univariates[j].value_at(i);
        }
        return result;
    };

    /**
     * @brief Compute the evaluation of a `relation` in different ways, comparing it to the provided `expected_evals`
     *
     * @details Check both `add_full_relation_value_contribution` and `add_edge_contribution` by comparing the result to
     * the `expected_evals` computed by the caller.
     * Ensures that the relations compute the same result as the expression given in the tests.
     *
     * @param expected_evals Relation evaluation computed by the caller.
     * @param relation being tested
     * @param extended_edges
     * @param relation_parameters
     */
    template <size_t FULL_RELATION_LENGTH, size_t NUM_POLYNOMIALS>
    static void validate_evaluations(
        const Univariate<FULL_RELATION_LENGTH>& expected_evals,
        const auto relation,
        const std::array<Univariate<FULL_RELATION_LENGTH>, NUM_POLYNOMIALS>& extended_edges,
        const RelationParameters<FF>& relation_parameters)
    {

        // Compute the expression index-by-index
        Univariate<FULL_RELATION_LENGTH> expected_evals_index{ 0 };
        for (size_t i = 0; i < FULL_RELATION_LENGTH; ++i) {
            // Get an array of the same size as `extended_edges` with only the i-th element of each extended edge.
            std::array evals_i = transposed_univariate_array_at(extended_edges, i);
            // Evaluate the relation
            relation.add_full_relation_value_contribution(
                expected_evals_index.value_at(i), evals_i, relation_parameters);
        }
        EXPECT_EQ(expected_evals, expected_evals_index);

        // Compute the expression using the class, that converts the extended edges to UnivariateView
        auto expected_evals_view = Univariate<relation.RELATION_LENGTH>(0);
        // The scaling factor is essentially 1 since we are working with degree 1 univariates
        relation.add_edge_contribution(expected_evals_view, extended_edges, relation_parameters, 1);

        // Tiny hack to reduce `expected_evals` to be of size `relation.RELATION_LENGTH`
        Univariate<relation.RELATION_LENGTH> expected_evals_restricted{ UnivariateView<relation.RELATION_LENGTH>(
            expected_evals) };
        EXPECT_EQ(expected_evals_restricted, expected_evals_view);
    };
};
using FieldTypes = testing::Types<barretenberg::fr>;
TYPED_TEST_SUITE(RelationConsistency, FieldTypes);

#define SUMCHECK_RELATION_TYPE_ALIASES using FF = TypeParam;

TYPED_TEST(RelationConsistency, ArithmeticRelation)
{
    SUMCHECK_RELATION_TYPE_ALIASES
    using MULTIVARIATE = honk::StandardArithmetization::POLYNOMIAL;

    static constexpr size_t FULL_RELATION_LENGTH = 5;
    static const size_t NUM_POLYNOMIALS = proof_system::honk::StandardArithmetization::NUM_POLYNOMIALS;

    const auto relation_parameters = TestFixture::compute_mock_relation_parameters();
    auto run_test = [&relation_parameters](bool is_random_input) {
        std::array<Univariate<FF, FULL_RELATION_LENGTH>, NUM_POLYNOMIALS> extended_edges;
        std::array<Univariate<FF, INPUT_UNIVARIATE_LENGTH>, NUM_POLYNOMIALS> input_polynomials;
        if (!is_random_input) {
            // evaluation form, i.e. input_univariate(0) = 1, input_univariate(1) = 2,.. The polynomial is x+1.
            for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
                input_polynomials[i] = Univariate<FF, INPUT_UNIVARIATE_LENGTH>({ 1, 2 });
            }
            extended_edges = TestFixture::template compute_mock_extended_edges<FULL_RELATION_LENGTH>(input_polynomials);
        } else {
            // input_univariates are random polynomials of degree one
            for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
                input_polynomials[i] =
                    Univariate<FF, INPUT_UNIVARIATE_LENGTH>({ FF::random_element(), FF::random_element() });
            }
            extended_edges = TestFixture::template compute_mock_extended_edges<FULL_RELATION_LENGTH>(input_polynomials);
        };
        auto relation = ArithmeticRelation<FF>();
        // Manually compute the expected edge contribution
        const auto& w_l = extended_edges[MULTIVARIATE::W_L];
        const auto& w_r = extended_edges[MULTIVARIATE::W_R];
        const auto& w_o = extended_edges[MULTIVARIATE::W_O];
        const auto& q_m = extended_edges[MULTIVARIATE::Q_M];
        const auto& q_l = extended_edges[MULTIVARIATE::Q_L];
        const auto& q_r = extended_edges[MULTIVARIATE::Q_R];
        const auto& q_o = extended_edges[MULTIVARIATE::Q_O];
        const auto& q_c = extended_edges[MULTIVARIATE::Q_C];

        // We first compute the evaluations using UnivariateViews, with the provided hard-coded formula.
        // Ensure that expression changes are detected.
        // expected_evals, length 4, extends to { { 5, 22, 57, 116, 205} } for input polynomial {1, 2}
        auto expected_evals = (q_m * w_r * w_l) + (q_r * w_r) + (q_l * w_l) + (q_o * w_o) + (q_c);
        TestFixture::template validate_evaluations(expected_evals, relation, extended_edges, relation_parameters);
    };
    run_test(/* is_random_input=*/true);
    run_test(/* is_random_input=*/false);
};

TYPED_TEST(RelationConsistency, GrandProductComputationRelation)
{
    SUMCHECK_RELATION_TYPE_ALIASES
    using MULTIVARIATE = honk::StandardArithmetization::POLYNOMIAL;

    static constexpr size_t FULL_RELATION_LENGTH = 5;
    static const size_t NUM_POLYNOMIALS = proof_system::honk::StandardArithmetization::NUM_POLYNOMIALS;

    const auto relation_parameters = TestFixture::compute_mock_relation_parameters();
    auto run_test = [&relation_parameters](bool is_random_input) {
        std::array<Univariate<FF, FULL_RELATION_LENGTH>, NUM_POLYNOMIALS> extended_edges;
        std::array<Univariate<FF, INPUT_UNIVARIATE_LENGTH>, NUM_POLYNOMIALS> input_polynomials;
        if (!is_random_input) {
            // evaluation form, i.e. input_univariate(0) = 1, input_univariate(1) = 2,.. The polynomial is x+1.
            for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
                input_polynomials[i] = Univariate<FF, INPUT_UNIVARIATE_LENGTH>({ 1, 2 });
            }
            extended_edges = TestFixture::template compute_mock_extended_edges<FULL_RELATION_LENGTH>(input_polynomials);
        } else {
            // input_univariates are random polynomials of degree one
            for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
                input_polynomials[i] =
                    Univariate<FF, INPUT_UNIVARIATE_LENGTH>({ FF::random_element(), FF::random_element() });
            }
            extended_edges = TestFixture::template compute_mock_extended_edges<FULL_RELATION_LENGTH>(input_polynomials);
        };
        auto relation = GrandProductComputationRelation<FF>();

        const auto& beta = relation_parameters.beta;
        const auto& gamma = relation_parameters.gamma;
        const auto& public_input_delta = relation_parameters.public_input_delta;
        // TODO(#225)(luke): Write a test that illustrates the following?
        // Note: the below z_perm_shift = X^2 will fail because it results in a relation of degree 2*1*1*1 = 5 which
        // cannot be represented by 5 points. Therefore when we do the calculation then barycentrically extend, we are
        // effectively exprapolating a 4th degree polynomial instead of the correct 5th degree poly
        // auto z_perm_shift = Univariate<FF, 5>({ 1, 4, 9, 16, 25 }); // X^2

        // Manually compute the expected edge contribution
        const auto& w_1 = extended_edges[MULTIVARIATE::W_L];
        const auto& w_2 = extended_edges[MULTIVARIATE::W_R];
        const auto& w_3 = extended_edges[MULTIVARIATE::W_O];
        const auto& sigma_1 = extended_edges[MULTIVARIATE::SIGMA_1];
        const auto& sigma_2 = extended_edges[MULTIVARIATE::SIGMA_2];
        const auto& sigma_3 = extended_edges[MULTIVARIATE::SIGMA_3];
        const auto& id_1 = extended_edges[MULTIVARIATE::ID_1];
        const auto& id_2 = extended_edges[MULTIVARIATE::ID_2];
        const auto& id_3 = extended_edges[MULTIVARIATE::ID_3];
        const auto& z_perm = extended_edges[MULTIVARIATE::Z_PERM];
        const auto& z_perm_shift = extended_edges[MULTIVARIATE::Z_PERM_SHIFT];
        const auto& lagrange_first = extended_edges[MULTIVARIATE::LAGRANGE_FIRST];
        const auto& lagrange_last = extended_edges[MULTIVARIATE::LAGRANGE_LAST];

        // We first compute the evaluations using UnivariateViews, with the provided hard-coded formula.
        // Ensure that expression changes are detected.
        // expected_evals in the below step { { 27, 250, 1029, 2916, 6655 } } - { { 27, 125, 343, 729, 1331 } }
        auto expected_evals = (z_perm + lagrange_first) * (w_1 + id_1 * beta + gamma) * (w_2 + id_2 * beta + gamma) *
                                  (w_3 + id_3 * beta + gamma) -
                              (z_perm_shift + lagrange_last * public_input_delta) * (w_1 + sigma_1 * beta + gamma) *
                                  (w_2 + sigma_2 * beta + gamma) * (w_3 + sigma_3 * beta + gamma);

        TestFixture::template validate_evaluations(expected_evals, relation, extended_edges, relation_parameters);
    };
    run_test(/* is_random_input=*/true);
    run_test(/* is_random_input=*/false);
};

TYPED_TEST(RelationConsistency, GrandProductInitializationRelation)
{
    SUMCHECK_RELATION_TYPE_ALIASES
    using MULTIVARIATE = honk::StandardArithmetization::POLYNOMIAL;

    static constexpr size_t FULL_RELATION_LENGTH = 5;
    static const size_t NUM_POLYNOMIALS = proof_system::honk::StandardArithmetization::NUM_POLYNOMIALS;

    const auto relation_parameters = TestFixture::compute_mock_relation_parameters();
    auto run_test = [&relation_parameters](bool is_random_input) {
        std::array<Univariate<FF, FULL_RELATION_LENGTH>, NUM_POLYNOMIALS> extended_edges;
        std::array<Univariate<FF, INPUT_UNIVARIATE_LENGTH>, NUM_POLYNOMIALS> input_polynomials;
        if (!is_random_input) {
            // evaluation form, i.e. input_univariate(0) = 1, input_univariate(1) = 2,.. The polynomial is x+1.
            for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
                input_polynomials[i] = Univariate<FF, INPUT_UNIVARIATE_LENGTH>({ 1, 2 });
            }
            extended_edges = TestFixture::template compute_mock_extended_edges<FULL_RELATION_LENGTH>(input_polynomials);
        } else {
            // input_univariates are random polynomials of degree one
            for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
                input_polynomials[i] =
                    Univariate<FF, INPUT_UNIVARIATE_LENGTH>({ FF::random_element(), FF::random_element() });
            }
            extended_edges = TestFixture::template compute_mock_extended_edges<FULL_RELATION_LENGTH>(input_polynomials);
        };
        auto relation = GrandProductInitializationRelation<FF>();
        const auto& z_perm_shift = extended_edges[MULTIVARIATE::Z_PERM_SHIFT];
        const auto& lagrange_last = extended_edges[MULTIVARIATE::LAGRANGE_LAST];
        // We first compute the evaluations using UnivariateViews, with the provided hard-coded formula.
        // Ensure that expression changes are detected.
        // expected_evals, lenght 3 (coeff form = x^2 + x), extends to { { 0, 2, 6, 12, 20 } }
        auto expected_evals = z_perm_shift * lagrange_last;

        TestFixture::template validate_evaluations(expected_evals, relation, extended_edges, relation_parameters);
    };
    run_test(/* is_random_input=*/true);
    run_test(/* is_random_input=*/false);
};

TYPED_TEST(RelationConsistency, UltraArithmeticRelation)
{
    SUMCHECK_RELATION_TYPE_ALIASES
    using MULTIVARIATE = honk::UltraArithmetization::POLYNOMIAL;

    static constexpr size_t FULL_RELATION_LENGTH = 6;
    static const size_t NUM_POLYNOMIALS = proof_system::honk::UltraArithmetization::COUNT;

    const auto relation_parameters = TestFixture::compute_mock_relation_parameters();
    std::array<Univariate<FF, FULL_RELATION_LENGTH>, NUM_POLYNOMIALS> extended_edges;
    std::array<Univariate<FF, INPUT_UNIVARIATE_LENGTH>, NUM_POLYNOMIALS> input_polynomials;

    // input_univariates are random polynomials of degree one
    for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
        input_polynomials[i] = Univariate<FF, INPUT_UNIVARIATE_LENGTH>({ FF::random_element(), FF::random_element() });
    }
    extended_edges = TestFixture::template compute_mock_extended_edges<FULL_RELATION_LENGTH>(input_polynomials);

    auto relation = UltraArithmeticRelation<FF>();

    // Extract the extended edges for manual computation of relation contribution
    const auto& w_1 = extended_edges[MULTIVARIATE::W_L];
    const auto& w_2 = extended_edges[MULTIVARIATE::W_R];
    const auto& w_3 = extended_edges[MULTIVARIATE::W_O];
    const auto& w_4 = extended_edges[MULTIVARIATE::W_4];
    const auto& w_4_shift = extended_edges[MULTIVARIATE::W_4_SHIFT];
    const auto& q_m = extended_edges[MULTIVARIATE::Q_M];
    const auto& q_l = extended_edges[MULTIVARIATE::Q_L];
    const auto& q_r = extended_edges[MULTIVARIATE::Q_R];
    const auto& q_o = extended_edges[MULTIVARIATE::Q_O];
    const auto& q_4 = extended_edges[MULTIVARIATE::Q_4];
    const auto& q_c = extended_edges[MULTIVARIATE::Q_C];
    const auto& q_arith = extended_edges[MULTIVARIATE::QARITH];

    static const FF neg_half = FF(-2).invert();

    auto expected_evals = (q_arith - 3) * (q_m * w_2 * w_1) * neg_half;
    expected_evals += (q_l * w_1) + (q_r * w_2) + (q_o * w_3) + (q_4 * w_4) + q_c;
    expected_evals += (q_arith - 1) * w_4_shift;
    expected_evals *= q_arith;

    TestFixture::template validate_evaluations(expected_evals, relation, extended_edges, relation_parameters);
};

TYPED_TEST(RelationConsistency, UltraArithmeticRelationSecondary)
{
    SUMCHECK_RELATION_TYPE_ALIASES
    using MULTIVARIATE = honk::UltraArithmetization::POLYNOMIAL;

    static constexpr size_t FULL_RELATION_LENGTH = 6;
    static const size_t NUM_POLYNOMIALS = proof_system::honk::UltraArithmetization::COUNT;

    const auto relation_parameters = TestFixture::compute_mock_relation_parameters();
    std::array<Univariate<FF, FULL_RELATION_LENGTH>, NUM_POLYNOMIALS> extended_edges;
    std::array<Univariate<FF, INPUT_UNIVARIATE_LENGTH>, NUM_POLYNOMIALS> input_polynomials;

    // input_univariates are random polynomials of degree one
    for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
        input_polynomials[i] = Univariate<FF, INPUT_UNIVARIATE_LENGTH>({ FF::random_element(), FF::random_element() });
    }
    extended_edges = TestFixture::template compute_mock_extended_edges<FULL_RELATION_LENGTH>(input_polynomials);

    auto relation = UltraArithmeticRelationSecondary<FF>();

    // Extract the extended edges for manual computation of relation contribution
    const auto& w_1 = extended_edges[MULTIVARIATE::W_L];
    const auto& w_4 = extended_edges[MULTIVARIATE::W_4];
    const auto& w_1_shift = extended_edges[MULTIVARIATE::W_1_SHIFT];
    const auto& q_m = extended_edges[MULTIVARIATE::Q_M];
    const auto& q_arith = extended_edges[MULTIVARIATE::QARITH];

    auto expected_evals = (w_1 + w_4 - w_1_shift + q_m);
    expected_evals *= (q_arith - 2) * (q_arith - 1) * q_arith;

    TestFixture::template validate_evaluations(expected_evals, relation, extended_edges, relation_parameters);
};

TYPED_TEST(RelationConsistency, UltraGrandProductInitializationRelation)
{
    SUMCHECK_RELATION_TYPE_ALIASES
    using MULTIVARIATE = honk::UltraArithmetization::POLYNOMIAL;

    static constexpr size_t FULL_RELATION_LENGTH = 6;
    static const size_t NUM_POLYNOMIALS = proof_system::honk::UltraArithmetization::COUNT;

    const auto relation_parameters = TestFixture::compute_mock_relation_parameters();
    std::array<Univariate<FF, FULL_RELATION_LENGTH>, NUM_POLYNOMIALS> extended_edges;
    std::array<Univariate<FF, INPUT_UNIVARIATE_LENGTH>, NUM_POLYNOMIALS> input_polynomials;

    // input_univariates are random polynomials of degree one
    for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
        input_polynomials[i] = Univariate<FF, INPUT_UNIVARIATE_LENGTH>({ FF::random_element(), FF::random_element() });
    }
    extended_edges = TestFixture::template compute_mock_extended_edges<FULL_RELATION_LENGTH>(input_polynomials);

    auto relation = UltraGrandProductInitializationRelation<FF>();

    // Extract the extended edges for manual computation of relation contribution
    const auto& z_perm_shift = extended_edges[MULTIVARIATE::Z_PERM_SHIFT];
    const auto& lagrange_last = extended_edges[MULTIVARIATE::LAGRANGE_LAST];

    // Compute the expected result using a simple to read version of the relation expression
    auto expected_evals = z_perm_shift * lagrange_last;

    TestFixture::template validate_evaluations(expected_evals, relation, extended_edges, relation_parameters);
};

TYPED_TEST(RelationConsistency, UltraGrandProductComputationRelation)
{
    SUMCHECK_RELATION_TYPE_ALIASES
    using MULTIVARIATE = honk::UltraArithmetization::POLYNOMIAL;

    static constexpr size_t FULL_RELATION_LENGTH = 6;
    static const size_t NUM_POLYNOMIALS = proof_system::honk::UltraArithmetization::COUNT;

    const auto relation_parameters = TestFixture::compute_mock_relation_parameters();
    std::array<Univariate<FF, FULL_RELATION_LENGTH>, NUM_POLYNOMIALS> extended_edges;
    std::array<Univariate<FF, INPUT_UNIVARIATE_LENGTH>, NUM_POLYNOMIALS> input_polynomials;

    // input_univariates are random polynomials of degree one
    for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
        input_polynomials[i] = Univariate<FF, INPUT_UNIVARIATE_LENGTH>({ FF::random_element(), FF::random_element() });
    }
    extended_edges = TestFixture::template compute_mock_extended_edges<FULL_RELATION_LENGTH>(input_polynomials);

    auto relation = UltraGrandProductComputationRelation<FF>();

    const auto& beta = relation_parameters.beta;
    const auto& gamma = relation_parameters.gamma;
    const auto& public_input_delta = relation_parameters.public_input_delta;

    // Extract the extended edges for manual computation of relation contribution
    const auto& w_1 = extended_edges[MULTIVARIATE::W_L];
    const auto& w_2 = extended_edges[MULTIVARIATE::W_R];
    const auto& w_3 = extended_edges[MULTIVARIATE::W_O];
    const auto& w_4 = extended_edges[MULTIVARIATE::W_4];
    const auto& sigma_1 = extended_edges[MULTIVARIATE::SIGMA_1];
    const auto& sigma_2 = extended_edges[MULTIVARIATE::SIGMA_2];
    const auto& sigma_3 = extended_edges[MULTIVARIATE::SIGMA_3];
    const auto& sigma_4 = extended_edges[MULTIVARIATE::SIGMA_4];
    const auto& id_1 = extended_edges[MULTIVARIATE::ID_1];
    const auto& id_2 = extended_edges[MULTIVARIATE::ID_2];
    const auto& id_3 = extended_edges[MULTIVARIATE::ID_3];
    const auto& id_4 = extended_edges[MULTIVARIATE::ID_4];
    const auto& z_perm = extended_edges[MULTIVARIATE::Z_PERM];
    const auto& z_perm_shift = extended_edges[MULTIVARIATE::Z_PERM_SHIFT];
    const auto& lagrange_first = extended_edges[MULTIVARIATE::LAGRANGE_FIRST];
    const auto& lagrange_last = extended_edges[MULTIVARIATE::LAGRANGE_LAST];

    // Compute the expected result using a simple to read version of the relation expression
    auto expected_evals = (z_perm + lagrange_first) * (w_1 + id_1 * beta + gamma) * (w_2 + id_2 * beta + gamma) *
                              (w_3 + id_3 * beta + gamma) * (w_4 + id_4 * beta + gamma) -
                          (z_perm_shift + lagrange_last * public_input_delta) * (w_1 + sigma_1 * beta + gamma) *
                              (w_2 + sigma_2 * beta + gamma) * (w_3 + sigma_3 * beta + gamma) *
                              (w_4 + sigma_4 * beta + gamma);

    TestFixture::template validate_evaluations(expected_evals, relation, extended_edges, relation_parameters);
};

TYPED_TEST(RelationConsistency, LookupGrandProductComputationRelation)
{
    SUMCHECK_RELATION_TYPE_ALIASES
    using MULTIVARIATE = honk::UltraArithmetization::POLYNOMIAL;

    static constexpr size_t FULL_RELATION_LENGTH = 6;
    static const size_t NUM_POLYNOMIALS = proof_system::honk::UltraArithmetization::COUNT;

    const auto relation_parameters = TestFixture::compute_mock_relation_parameters();
    std::array<Univariate<FF, FULL_RELATION_LENGTH>, NUM_POLYNOMIALS> extended_edges;
    std::array<Univariate<FF, INPUT_UNIVARIATE_LENGTH>, NUM_POLYNOMIALS> input_polynomials;

    // input_univariates are random polynomials of degree one
    for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
        input_polynomials[i] = Univariate<FF, INPUT_UNIVARIATE_LENGTH>({ FF::random_element(), FF::random_element() });
    }
    extended_edges = TestFixture::template compute_mock_extended_edges<FULL_RELATION_LENGTH>(input_polynomials);

    auto relation = LookupGrandProductComputationRelation<FF>();

    const auto eta = relation_parameters.eta;
    const auto beta = relation_parameters.beta;
    const auto gamma = relation_parameters.gamma;
    auto grand_product_delta = relation_parameters.lookup_grand_product_delta;

    // Extract the extended edges for manual computation of relation contribution
    auto one_plus_beta = FF::one() + beta;
    auto gamma_by_one_plus_beta = gamma * one_plus_beta;
    auto eta_sqr = eta * eta;
    auto eta_cube = eta_sqr * eta;

    const auto& w_1 = extended_edges[MULTIVARIATE::W_L];
    const auto& w_2 = extended_edges[MULTIVARIATE::W_R];
    const auto& w_3 = extended_edges[MULTIVARIATE::W_O];

    const auto& w_1_shift = extended_edges[MULTIVARIATE::W_1_SHIFT];
    const auto& w_2_shift = extended_edges[MULTIVARIATE::W_2_SHIFT];
    const auto& w_3_shift = extended_edges[MULTIVARIATE::W_3_SHIFT];

    const auto& table_1 = extended_edges[MULTIVARIATE::TABLE_1];
    const auto& table_2 = extended_edges[MULTIVARIATE::TABLE_2];
    const auto& table_3 = extended_edges[MULTIVARIATE::TABLE_3];
    const auto& table_4 = extended_edges[MULTIVARIATE::TABLE_4];

    const auto& table_1_shift = extended_edges[MULTIVARIATE::TABLE_1_SHIFT];
    const auto& table_2_shift = extended_edges[MULTIVARIATE::TABLE_2_SHIFT];
    const auto& table_3_shift = extended_edges[MULTIVARIATE::TABLE_3_SHIFT];
    const auto& table_4_shift = extended_edges[MULTIVARIATE::TABLE_4_SHIFT];

    const auto& s_accum = extended_edges[MULTIVARIATE::S_ACCUM];
    const auto& s_accum_shift = extended_edges[MULTIVARIATE::S_ACCUM_SHIFT];
    const auto& z_lookup = extended_edges[MULTIVARIATE::Z_LOOKUP];
    const auto& z_lookup_shift = extended_edges[MULTIVARIATE::Z_LOOKUP_SHIFT];

    const auto& table_index = extended_edges[MULTIVARIATE::Q_O];
    const auto& column_1_step_size = extended_edges[MULTIVARIATE::Q_R];
    const auto& column_2_step_size = extended_edges[MULTIVARIATE::Q_M];
    const auto& column_3_step_size = extended_edges[MULTIVARIATE::Q_C];
    const auto& q_lookup = extended_edges[MULTIVARIATE::QLOOKUPTYPE];

    const auto& lagrange_first = extended_edges[MULTIVARIATE::LAGRANGE_FIRST];
    const auto& lagrange_last = extended_edges[MULTIVARIATE::LAGRANGE_LAST];

    auto wire_accum = (w_1 + column_1_step_size * w_1_shift) + (w_2 + column_2_step_size * w_2_shift) * eta +
                      (w_3 + column_3_step_size * w_3_shift) * eta_sqr + table_index * eta_cube;

    auto table_accum = table_1 + table_2 * eta + table_3 * eta_sqr + table_4 * eta_cube;
    auto table_accum_shift = table_1_shift + table_2_shift * eta + table_3_shift * eta_sqr + table_4_shift * eta_cube;

    // Compute the expected result using a simple to read version of the relation expression
    auto expected_evals = (z_lookup + lagrange_first) * (q_lookup * wire_accum + gamma) *
                          (table_accum + table_accum_shift * beta + gamma_by_one_plus_beta) * one_plus_beta;
    expected_evals -= (z_lookup_shift + lagrange_last * grand_product_delta) *
                      (s_accum + s_accum_shift * beta + gamma_by_one_plus_beta);

    TestFixture::template validate_evaluations(expected_evals, relation, extended_edges, relation_parameters);
};

TYPED_TEST(RelationConsistency, LookupGrandProductInitializationRelation)
{
    SUMCHECK_RELATION_TYPE_ALIASES
    using MULTIVARIATE = honk::UltraArithmetization::POLYNOMIAL;

    static constexpr size_t FULL_RELATION_LENGTH = 6;
    static const size_t NUM_POLYNOMIALS = proof_system::honk::UltraArithmetization::COUNT;

    const auto relation_parameters = TestFixture::compute_mock_relation_parameters();
    std::array<Univariate<FF, FULL_RELATION_LENGTH>, NUM_POLYNOMIALS> extended_edges;
    std::array<Univariate<FF, INPUT_UNIVARIATE_LENGTH>, NUM_POLYNOMIALS> input_polynomials;

    // input_univariates are random polynomials of degree one
    for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
        input_polynomials[i] = Univariate<FF, INPUT_UNIVARIATE_LENGTH>({ FF::random_element(), FF::random_element() });
    }
    extended_edges = TestFixture::template compute_mock_extended_edges<FULL_RELATION_LENGTH>(input_polynomials);

    auto relation = LookupGrandProductInitializationRelation<FF>();

    // Extract the extended edges for manual computation of relation contribution
    const auto& z_lookup_shift = extended_edges[MULTIVARIATE::Z_LOOKUP_SHIFT];
    const auto& lagrange_last = extended_edges[MULTIVARIATE::LAGRANGE_LAST];

    // Compute the expected result using a simple to read version of the relation expression
    auto expected_evals = z_lookup_shift * lagrange_last;

    TestFixture::template validate_evaluations(expected_evals, relation, extended_edges, relation_parameters);
};

} // namespace proof_system::honk_relation_tests
