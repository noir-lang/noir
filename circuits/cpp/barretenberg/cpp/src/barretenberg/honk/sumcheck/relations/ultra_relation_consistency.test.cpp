#include "barretenberg/honk/sumcheck/relations/ultra_arithmetic_relation.hpp"
#include "barretenberg/honk/sumcheck/relations/ultra_arithmetic_relation_secondary.hpp"
#include "barretenberg/honk/sumcheck/relations/lookup_grand_product_relation.hpp"
#include "barretenberg/honk/flavor/ultra.hpp"
#include "relation.hpp"
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

class UltraRelationConsistency : public testing::Test {
  public:
    using Flavor = honk::flavor::Ultra;
    using FF = typename Flavor::FF;
    using PurportedEvaluations = typename Flavor::PurportedEvaluations;

    // TODO(#390): Move MAX_RELATION_LENGTH into Flavor and simplify this.
    template <size_t t> using ExtendedEdges = typename Flavor::template ExtendedEdges<t>;

    // TODO(#225)(Adrian): Accept FULL_RELATION_LENGTH as a template parameter for this function only, so that the test
    // can decide to which degree the polynomials must be extended. Possible accept an existing list of "edges" and
    // extend them to the degree.
    template <size_t FULL_RELATION_LENGTH, size_t NUM_POLYNOMIALS>
    static void compute_mock_extended_edges(
        ExtendedEdges<FULL_RELATION_LENGTH>& extended_edges,
        std::array<Univariate<FF, INPUT_UNIVARIATE_LENGTH>, NUM_POLYNOMIALS>& input_edges)
    {
        BarycentricData<FF, INPUT_UNIVARIATE_LENGTH, FULL_RELATION_LENGTH> barycentric_2_to_max =
            BarycentricData<FF, INPUT_UNIVARIATE_LENGTH, FULL_RELATION_LENGTH>();
        for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
            extended_edges[i] = barycentric_2_to_max.extend(input_edges[i]);
        }
    }

    /**
     * @brief Returns randomly sampled parameters to feed to the relations.
     *
     * @return RelationParameters<FF>
     */
    RelationParameters<FF> compute_mock_relation_parameters()
    {
        return { .beta = FF::random_element(),
                 .gamma = FF::random_element(),
                 .public_input_delta = FF::random_element() };
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
    template <size_t univariate_length>
    static PurportedEvaluations transposed_univariate_array_at(ExtendedEdges<univariate_length> univariates, size_t i)
    {
        ASSERT(i < univariate_length);
        std::array<FF, Flavor::NUM_ALL_ENTITIES> result;
        size_t result_idx = 0; // TODO(#391) zip
        for (auto& univariate : univariates) {
            result[result_idx] = univariate.value_at(i);
            ++result_idx;
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
    template <size_t FULL_RELATION_LENGTH>
    static void validate_evaluations(const Univariate<FF, FULL_RELATION_LENGTH>& expected_evals,
                                     const auto relation,
                                     const ExtendedEdges<FULL_RELATION_LENGTH>& extended_edges,
                                     const RelationParameters<FF>& relation_parameters)
    {

        // Compute the expression index-by-index
        Univariate<FF, FULL_RELATION_LENGTH> expected_evals_index{ 0 };
        for (size_t i = 0; i < FULL_RELATION_LENGTH; ++i) {
            // Get an array of the same size as `extended_edges` with only the i-th element of each extended edge.
            PurportedEvaluations evals_i = transposed_univariate_array_at(extended_edges, i);
            // Evaluate the relation
            relation.add_full_relation_value_contribution(
                expected_evals_index.value_at(i), evals_i, relation_parameters);
        }
        EXPECT_EQ(expected_evals, expected_evals_index);

        // Compute the expression using the class, that converts the extended edges to UnivariateView
        auto expected_evals_view = Univariate<FF, relation.RELATION_LENGTH>(0);
        // The scaling factor is essentially 1 since we are working with degree 1 univariates
        relation.add_edge_contribution(expected_evals_view, extended_edges, relation_parameters, 1);

        // Tiny hack to reduce `expected_evals` to be of size `relation.RELATION_LENGTH`
        Univariate<FF, relation.RELATION_LENGTH> expected_evals_restricted{
            UnivariateView<FF, relation.RELATION_LENGTH>(expected_evals)
        };
        EXPECT_EQ(expected_evals_restricted, expected_evals_view);
    };
};

TEST_F(UltraRelationConsistency, UltraArithmeticRelation)
{
    using Flavor = honk::flavor::Ultra;
    using FF = typename Flavor::FF;
    static constexpr size_t FULL_RELATION_LENGTH = 6;
    using ExtendedEdges = typename Flavor::template ExtendedEdges<FULL_RELATION_LENGTH>;
    static const size_t NUM_POLYNOMIALS = Flavor::NUM_ALL_ENTITIES;

    const auto relation_parameters = compute_mock_relation_parameters();
    ExtendedEdges extended_edges;
    std::array<Univariate<FF, INPUT_UNIVARIATE_LENGTH>, NUM_POLYNOMIALS> input_polynomials;

    // input_univariates are random polynomials of degree one
    for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
        input_polynomials[i] = Univariate<FF, INPUT_UNIVARIATE_LENGTH>({ FF::random_element(), FF::random_element() });
    }
    compute_mock_extended_edges<FULL_RELATION_LENGTH>(extended_edges, input_polynomials);

    auto relation = UltraArithmeticRelation<FF>();

    // Extract the extended edges for manual computation of relation contribution
    const auto& w_1 = extended_edges.w_l;
    const auto& w_2 = extended_edges.w_r;
    const auto& w_3 = extended_edges.w_o;
    const auto& w_4 = extended_edges.w_4;
    const auto& w_4_shift = extended_edges.w_4_shift;
    const auto& q_m = extended_edges.q_m;
    const auto& q_l = extended_edges.q_l;
    const auto& q_r = extended_edges.q_r;
    const auto& q_o = extended_edges.q_o;
    const auto& q_4 = extended_edges.q_4;
    const auto& q_c = extended_edges.q_c;
    const auto& q_arith = extended_edges.q_arith;

    static const FF neg_half = FF(-2).invert();

    auto expected_evals = (q_arith - 3) * (q_m * w_2 * w_1) * neg_half;
    expected_evals += (q_l * w_1) + (q_r * w_2) + (q_o * w_3) + (q_4 * w_4) + q_c;
    expected_evals += (q_arith - 1) * w_4_shift;
    expected_evals *= q_arith;

    validate_evaluations(expected_evals, relation, extended_edges, relation_parameters);
};

TEST_F(UltraRelationConsistency, UltraArithmeticRelationSecondary)
{
    using Flavor = honk::flavor::Ultra;
    using FF = typename Flavor::FF;
    static constexpr size_t FULL_RELATION_LENGTH = 6;
    using ExtendedEdges = typename Flavor::template ExtendedEdges<FULL_RELATION_LENGTH>;
    static const size_t NUM_POLYNOMIALS = Flavor::NUM_ALL_ENTITIES;

    const auto relation_parameters = compute_mock_relation_parameters();
    ExtendedEdges extended_edges;
    std::array<Univariate<FF, INPUT_UNIVARIATE_LENGTH>, NUM_POLYNOMIALS> input_polynomials;

    // input_univariates are random polynomials of degree one
    for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
        input_polynomials[i] = Univariate<FF, INPUT_UNIVARIATE_LENGTH>({ FF::random_element(), FF::random_element() });
    }
    compute_mock_extended_edges<FULL_RELATION_LENGTH>(extended_edges, input_polynomials);

    auto relation = UltraArithmeticRelationSecondary<FF>();

    // Extract the extended edges for manual computation of relation contribution
    const auto& w_1 = extended_edges.w_l;
    const auto& w_4 = extended_edges.w_4;
    const auto& w_l_shift = extended_edges.w_l_shift;
    const auto& q_m = extended_edges.q_m;
    const auto& q_arith = extended_edges.q_arith;

    auto expected_evals = (w_1 + w_4 - w_l_shift + q_m);
    expected_evals *= (q_arith - 2) * (q_arith - 1) * q_arith;

    validate_evaluations(expected_evals, relation, extended_edges, relation_parameters);
};

TEST_F(UltraRelationConsistency, UltraGrandProductInitializationRelation)
{
    using Flavor = honk::flavor::Ultra;
    using FF = typename Flavor::FF;
    using Flavor = honk::flavor::Ultra;
    static constexpr size_t FULL_RELATION_LENGTH = 6;
    using ExtendedEdges = typename Flavor::ExtendedEdges<FULL_RELATION_LENGTH>;
    static const size_t NUM_POLYNOMIALS = Flavor::NUM_ALL_ENTITIES;
    auto relation_parameters = compute_mock_relation_parameters();
    ExtendedEdges extended_edges;
    std::array<Univariate<FF, INPUT_UNIVARIATE_LENGTH>, NUM_POLYNOMIALS> input_polynomials;

    // input_univariates are random polynomials of degree one
    for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
        input_polynomials[i] = Univariate<FF, INPUT_UNIVARIATE_LENGTH>({ FF::random_element(), FF::random_element() });
    }
    compute_mock_extended_edges(extended_edges, input_polynomials);

    auto relation = UltraGrandProductInitializationRelation<FF>();

    // Extract the extended edges for manual computation of relation contribution
    const auto& z_perm_shift = extended_edges.z_perm_shift;
    const auto& lagrange_last = extended_edges.lagrange_last;

    // Compute the expected result using a simple to read version of the relation expression
    auto expected_evals = z_perm_shift * lagrange_last;

    validate_evaluations(expected_evals, relation, extended_edges, relation_parameters);
};

TEST_F(UltraRelationConsistency, UltraGrandProductComputationRelation)
{
    using Flavor = honk::flavor::Ultra;
    using FF = typename Flavor::FF;
    using Flavor = honk::flavor::Ultra;
    static constexpr size_t FULL_RELATION_LENGTH = 6;
    using ExtendedEdges = typename Flavor::template ExtendedEdges<FULL_RELATION_LENGTH>;
    static const size_t NUM_POLYNOMIALS = Flavor::NUM_ALL_ENTITIES;
    auto relation_parameters = compute_mock_relation_parameters();
    ExtendedEdges extended_edges;
    std::array<Univariate<FF, INPUT_UNIVARIATE_LENGTH>, NUM_POLYNOMIALS> input_polynomials;

    // input_univariates are random polynomials of degree one
    for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
        input_polynomials[i] = Univariate<FF, INPUT_UNIVARIATE_LENGTH>({ FF::random_element(), FF::random_element() });
    }
    compute_mock_extended_edges(extended_edges, input_polynomials);

    auto relation = UltraGrandProductComputationRelation<FF>();

    const auto& beta = relation_parameters.beta;
    const auto& gamma = relation_parameters.gamma;
    const auto& public_input_delta = relation_parameters.public_input_delta;

    // Extract the extended edges for manual computation of relation contribution
    const auto& w_1 = extended_edges.w_l;
    const auto& w_2 = extended_edges.w_r;
    const auto& w_3 = extended_edges.w_o;
    const auto& w_4 = extended_edges.w_4;
    const auto& sigma_1 = extended_edges.sigma_1;
    const auto& sigma_2 = extended_edges.sigma_2;
    const auto& sigma_3 = extended_edges.sigma_3;
    const auto& sigma_4 = extended_edges.sigma_4;
    const auto& id_1 = extended_edges.id_1;
    const auto& id_2 = extended_edges.id_2;
    const auto& id_3 = extended_edges.id_3;
    const auto& id_4 = extended_edges.id_4;
    const auto& z_perm = extended_edges.z_perm;
    const auto& z_perm_shift = extended_edges.z_perm_shift;
    const auto& lagrange_first = extended_edges.lagrange_first;
    const auto& lagrange_last = extended_edges.lagrange_last;

    // Compute the expected result using a simple to read version of the relation expression
    auto expected_evals = (z_perm + lagrange_first) * (w_1 + id_1 * beta + gamma) * (w_2 + id_2 * beta + gamma) *
                              (w_3 + id_3 * beta + gamma) * (w_4 + id_4 * beta + gamma) -
                          (z_perm_shift + lagrange_last * public_input_delta) * (w_1 + sigma_1 * beta + gamma) *
                              (w_2 + sigma_2 * beta + gamma) * (w_3 + sigma_3 * beta + gamma) *
                              (w_4 + sigma_4 * beta + gamma);

    validate_evaluations(expected_evals, relation, extended_edges, relation_parameters);
};

TEST_F(UltraRelationConsistency, LookupGrandProductComputationRelation)
{
    using Flavor = honk::flavor::Ultra;
    using FF = typename Flavor::FF;
    using Flavor = honk::flavor::Ultra;
    static constexpr size_t FULL_RELATION_LENGTH = 6;
    using ExtendedEdges = typename Flavor::ExtendedEdges<FULL_RELATION_LENGTH>;
    static const size_t NUM_POLYNOMIALS = Flavor::NUM_ALL_ENTITIES;
    auto relation_parameters = compute_mock_relation_parameters();
    ExtendedEdges extended_edges;
    std::array<Univariate<FF, INPUT_UNIVARIATE_LENGTH>, NUM_POLYNOMIALS> input_polynomials;

    // input_univariates are random polynomials of degree one
    for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
        input_polynomials[i] = Univariate<FF, INPUT_UNIVARIATE_LENGTH>({ FF::random_element(), FF::random_element() });
    }
    compute_mock_extended_edges(extended_edges, input_polynomials);

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

    const auto& w_1 = extended_edges.w_l;
    const auto& w_2 = extended_edges.w_r;
    const auto& w_3 = extended_edges.w_o;

    const auto& w_1_shift = extended_edges.w_l_shift;
    const auto& w_2_shift = extended_edges.w_r_shift;
    const auto& w_3_shift = extended_edges.w_o_shift;

    const auto& table_1 = extended_edges.table_1;
    const auto& table_2 = extended_edges.table_2;
    const auto& table_3 = extended_edges.table_3;
    const auto& table_4 = extended_edges.table_4;

    const auto& table_1_shift = extended_edges.table_1_shift;
    const auto& table_2_shift = extended_edges.table_2_shift;
    const auto& table_3_shift = extended_edges.table_3_shift;
    const auto& table_4_shift = extended_edges.table_4_shift;

    const auto& s_accum = extended_edges.sorted_accum;
    const auto& s_accum_shift = extended_edges.sorted_accum_shift;
    const auto& z_lookup = extended_edges.z_lookup;
    const auto& z_lookup_shift = extended_edges.z_lookup_shift;

    const auto& table_index = extended_edges.q_o;
    const auto& column_1_step_size = extended_edges.q_r;
    const auto& column_2_step_size = extended_edges.q_m;
    const auto& column_3_step_size = extended_edges.q_c;
    const auto& q_lookup = extended_edges.q_lookup;

    const auto& lagrange_first = extended_edges.lagrange_first;
    const auto& lagrange_last = extended_edges.lagrange_last;

    auto wire_accum = (w_1 + column_1_step_size * w_1_shift) + (w_2 + column_2_step_size * w_2_shift) * eta +
                      (w_3 + column_3_step_size * w_3_shift) * eta_sqr + table_index * eta_cube;

    auto table_accum = table_1 + table_2 * eta + table_3 * eta_sqr + table_4 * eta_cube;
    auto table_accum_shift = table_1_shift + table_2_shift * eta + table_3_shift * eta_sqr + table_4_shift * eta_cube;

    // Compute the expected result using a simple to read version of the relation expression
    auto expected_evals = (z_lookup + lagrange_first) * (q_lookup * wire_accum + gamma) *
                          (table_accum + table_accum_shift * beta + gamma_by_one_plus_beta) * one_plus_beta;
    expected_evals -= (z_lookup_shift + lagrange_last * grand_product_delta) *
                      (s_accum + s_accum_shift * beta + gamma_by_one_plus_beta);

    validate_evaluations(expected_evals, relation, extended_edges, relation_parameters);
};

TEST_F(UltraRelationConsistency, LookupGrandProductInitializationRelation)
{
    using Flavor = honk::flavor::Ultra;
    using FF = typename Flavor::FF;
    using Flavor = honk::flavor::Ultra;
    static constexpr size_t FULL_RELATION_LENGTH = 6;
    using ExtendedEdges = typename Flavor::ExtendedEdges<FULL_RELATION_LENGTH>;
    static const size_t NUM_POLYNOMIALS = Flavor::NUM_ALL_ENTITIES;
    auto relation_parameters = compute_mock_relation_parameters();
    ExtendedEdges extended_edges;
    std::array<Univariate<FF, INPUT_UNIVARIATE_LENGTH>, NUM_POLYNOMIALS> input_polynomials;

    // input_univariates are random polynomials of degree one
    for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
        input_polynomials[i] = Univariate<FF, INPUT_UNIVARIATE_LENGTH>({ FF::random_element(), FF::random_element() });
    }
    compute_mock_extended_edges(extended_edges, input_polynomials);

    auto relation = LookupGrandProductInitializationRelation<FF>();

    // Extract the extended edges for manual computation of relation contribution
    const auto& z_lookup_shift = extended_edges.z_lookup_shift;
    const auto& lagrange_last = extended_edges.lagrange_last;

    // Compute the expected result using a simple to read version of the relation expression
    auto expected_evals = z_lookup_shift * lagrange_last;

    validate_evaluations(expected_evals, relation, extended_edges, relation_parameters);
};

} // namespace proof_system::honk_relation_tests
