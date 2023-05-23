#include "barretenberg/honk/sumcheck/relations/ultra_arithmetic_relation.hpp"
#include "barretenberg/honk/sumcheck/relations/ultra_arithmetic_relation_secondary.hpp"
#include "barretenberg/honk/sumcheck/relations/lookup_grand_product_relation.hpp"
#include "barretenberg/honk/sumcheck/relations/gen_perm_sort_relation.hpp"
#include "barretenberg/honk/sumcheck/relations/elliptic_relation.hpp"
#include "barretenberg/honk/sumcheck/relations/auxiliary_relation.hpp"
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
    using ClaimedEvaluations = typename Flavor::ClaimedEvaluations;

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
    static ClaimedEvaluations transposed_univariate_array_at(ExtendedEdges<univariate_length> univariates, size_t i)
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
            ClaimedEvaluations evals_i = transposed_univariate_array_at(extended_edges, i);
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

TEST_F(UltraRelationConsistency, GenPermSortRelation)
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

    auto relation = GenPermSortRelation<FF>();

    // Extract the extended edges for manual computation of relation contribution
    const auto& z_lookup_shift = extended_edges.z_lookup_shift;
    const auto& lagrange_last = extended_edges.lagrange_last;

    const auto& w_1 = extended_edges.w_l;
    const auto& w_2 = extended_edges.w_r;
    const auto& w_3 = extended_edges.w_o;
    const auto& w_4 = extended_edges.w_4;
    const auto& w_1_shift = extended_edges.w_l_shift;
    const auto& q_sort = extended_edges.q_sort;

    static const FF fake_alpha_1 = FF(1);
    static const FF fake_alpha_2 = fake_alpha_1 * fake_alpha_1;
    static const FF fake_alpha_3 = fake_alpha_2 * fake_alpha_1;
    static const FF fake_alpha_4 = fake_alpha_3 * fake_alpha_1;

    // Compute wire differences
    auto delta_1 = w_2 - w_1;
    auto delta_2 = w_3 - w_2;
    auto delta_3 = w_4 - w_3;
    auto delta_4 = w_1_shift - w_4;

    // Compute the expected result using a simple to read version of the relation expression
    auto expected_evals = delta_1 * (delta_1 - 1) * (delta_1 - 2) * (delta_1 - 3) * fake_alpha_1;
    expected_evals += delta_2 * (delta_2 - 1) * (delta_2 - 2) * (delta_2 - 3) * fake_alpha_2;
    expected_evals += delta_3 * (delta_3 - 1) * (delta_3 - 2) * (delta_3 - 3) * fake_alpha_3;
    expected_evals += delta_4 * (delta_4 - 1) * (delta_4 - 2) * (delta_4 - 3) * fake_alpha_4;
    expected_evals *= q_sort;

    validate_evaluations(expected_evals, relation, extended_edges, relation_parameters);
};

TEST_F(UltraRelationConsistency, EllipticRelation)
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

    auto relation = EllipticRelation<FF>();

    // Extract the extended edges for manual computation of relation contribution
    const auto& x_1 = extended_edges.w_r;
    const auto& y_1 = extended_edges.w_o;

    const auto& x_2 = extended_edges.w_l_shift;
    const auto& y_2 = extended_edges.w_4_shift;
    const auto& x_3 = extended_edges.w_r_shift;
    const auto& y_3 = extended_edges.w_o_shift;

    const auto& q_sign = extended_edges.q_l;
    const auto& q_beta = extended_edges.q_o;
    const auto& q_beta_sqr = extended_edges.q_4;
    const auto& q_elliptic = extended_edges.q_elliptic;

    static const FF fake_alpha_1 = FF(1);
    static const FF fake_alpha_2 = fake_alpha_1 * fake_alpha_1;

    // Compute x/y coordinate identities
    auto x_identity = q_sign * (y_1 * y_2 * 2);
    x_identity += q_beta * (x_1 * x_2 * x_3 * 2 + x_1 * x_1 * x_2) * FF(-1);
    x_identity += q_beta_sqr * (x_2 * x_2 * x_3 - x_1 * x_2 * x_2);
    x_identity += (x_1 * x_1 * x_3 - y_2 * y_2 - y_1 * y_1 + x_2 * x_2 * x_2 + x_1 * x_1 * x_1);

    auto y_identity = q_sign * (y_2 * x_3 - y_2 * x_1);
    y_identity += q_beta * (x_2 * y_3 + y_1 * x_2);
    y_identity += (x_1 * y_1 - x_1 * y_3 - y_1 * x_3 - x_1 * y_1);

    auto expected_evals = x_identity * fake_alpha_1 + y_identity * fake_alpha_2;
    expected_evals *= q_elliptic;

    validate_evaluations(expected_evals, relation, extended_edges, relation_parameters);
};

TEST_F(UltraRelationConsistency, AuxiliaryRelation)
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

    auto relation = AuxiliaryRelation<FF>();

    const auto& eta = relation_parameters.eta;
    const auto fake_alpha = FF(1);

    // Extract the extended edges for manual computation of relation contribution
    const auto& w_1 = extended_edges.w_l;
    const auto& w_2 = extended_edges.w_r;
    const auto& w_3 = extended_edges.w_o;
    const auto& w_4 = extended_edges.w_4;
    const auto& w_1_shift = extended_edges.w_l_shift;
    const auto& w_2_shift = extended_edges.w_r_shift;
    const auto& w_3_shift = extended_edges.w_o_shift;
    const auto& w_4_shift = extended_edges.w_4_shift;

    const auto& q_1 = extended_edges.q_l;
    const auto& q_2 = extended_edges.q_r;
    const auto& q_3 = extended_edges.q_o;
    const auto& q_4 = extended_edges.q_4;
    const auto& q_m = extended_edges.q_m;
    const auto& q_c = extended_edges.q_c;
    const auto& q_arith = extended_edges.q_arith;
    const auto& q_aux = extended_edges.q_aux;

    constexpr FF LIMB_SIZE(uint256_t(1) << 68);
    constexpr FF SUBLIMB_SHIFT(uint256_t(1) << 14);
    constexpr FF SUBLIMB_SHIFT_2(SUBLIMB_SHIFT * SUBLIMB_SHIFT);
    constexpr FF SUBLIMB_SHIFT_3(SUBLIMB_SHIFT_2 * SUBLIMB_SHIFT);
    constexpr FF SUBLIMB_SHIFT_4(SUBLIMB_SHIFT_3 * SUBLIMB_SHIFT);

    /**
     * Non native field arithmetic gate 2
     *
     *             _                                                                               _
     *            /   _                   _                               _       14                \
     * q_2 . q_4 |   (w_1 . w_2) + (w_1 . w_2) + (w_1 . w_4 + w_2 . w_3 - w_3) . 2    - w_3 - w_4   |
     *            \_                                                                               _/
     *
     **/
    auto limb_subproduct = w_1 * w_2_shift + w_1_shift * w_2;
    auto non_native_field_gate_2 = (w_1 * w_4 + w_2 * w_3 - w_3_shift);
    non_native_field_gate_2 *= LIMB_SIZE;
    non_native_field_gate_2 -= w_4_shift;
    non_native_field_gate_2 += limb_subproduct;
    non_native_field_gate_2 *= q_4;

    limb_subproduct *= LIMB_SIZE;
    limb_subproduct += (w_1_shift * w_2_shift);
    auto non_native_field_gate_1 = limb_subproduct;
    non_native_field_gate_1 -= (w_3 + w_4);
    non_native_field_gate_1 *= q_3;

    auto non_native_field_gate_3 = limb_subproduct;
    non_native_field_gate_3 += w_4;
    non_native_field_gate_3 -= (w_3_shift + w_4_shift);
    non_native_field_gate_3 *= q_m;

    auto non_native_field_identity = non_native_field_gate_1 + non_native_field_gate_2 + non_native_field_gate_3;
    non_native_field_identity *= q_2;

    auto limb_accumulator_1 = w_1 + w_2 * SUBLIMB_SHIFT + w_3 * SUBLIMB_SHIFT_2 + w_1_shift * SUBLIMB_SHIFT_3 +
                              w_2_shift * SUBLIMB_SHIFT_4 - w_4;
    limb_accumulator_1 *= q_4;

    auto limb_accumulator_2 = w_3 + w_4 * SUBLIMB_SHIFT + w_1_shift * SUBLIMB_SHIFT_2 + w_2_shift * SUBLIMB_SHIFT_3 +
                              w_3_shift * SUBLIMB_SHIFT_4 - w_4_shift;
    limb_accumulator_2 *= q_m;

    auto limb_accumulator_identity = limb_accumulator_1 + limb_accumulator_2;
    limb_accumulator_identity *= q_3;

    /**
     * MEMORY
     **/

    /**
     * Memory Record Check
     */
    auto memory_record_check = w_3;
    memory_record_check *= eta;
    memory_record_check += w_2;
    memory_record_check *= eta;
    memory_record_check += w_1;
    memory_record_check *= eta;
    memory_record_check += q_c;
    auto partial_record_check = memory_record_check; // used in RAM consistency check
    memory_record_check = memory_record_check - w_4;

    /**
     * ROM Consistency Check
     */
    auto index_delta = w_1_shift - w_1;
    auto record_delta = w_4_shift - w_4;

    auto index_is_monotonically_increasing = index_delta * index_delta - index_delta;

    // auto adjacent_values_match_if_adjacent_indices_match = (FF(1) - index_delta) * record_delta;
    auto adjacent_values_match_if_adjacent_indices_match = (index_delta * FF(-1) + FF(1)) * record_delta;

    auto ROM_consistency_check_identity = adjacent_values_match_if_adjacent_indices_match;
    ROM_consistency_check_identity *= fake_alpha;
    ROM_consistency_check_identity += index_is_monotonically_increasing;
    ROM_consistency_check_identity *= fake_alpha;
    ROM_consistency_check_identity += memory_record_check;

    /**
     * RAM Consistency Check
     */
    auto access_type = (w_4 - partial_record_check);             // will be 0 or 1 for honest Prover
    auto access_check = access_type * access_type - access_type; // check value is 0 or 1

    auto next_gate_access_type = w_3_shift;
    next_gate_access_type *= eta;
    next_gate_access_type += w_2_shift;
    next_gate_access_type *= eta;
    next_gate_access_type += w_1_shift;
    next_gate_access_type *= eta;
    next_gate_access_type = w_4_shift - next_gate_access_type;

    auto value_delta = w_3_shift - w_3;
    auto adjacent_values_match_if_adjacent_indices_match_and_next_access_is_a_read_operation =
        (index_delta * FF(-1) + FF(1)) * value_delta * (next_gate_access_type * FF(-1) + FF(1));

    // We can't apply the RAM consistency check identity on the final entry in the sorted list (the wires in the
    // next gate would make the identity fail).
    // We need to validate that its 'access type' bool is correct. Can't do
    // with an arithmetic gate because of the `eta` factors. We need to check that the *next* gate's access type is
    // correct, to cover this edge case
    auto next_gate_access_type_is_boolean = next_gate_access_type * next_gate_access_type - next_gate_access_type;

    // Putting it all together...
    auto RAM_consistency_check_identity =
        adjacent_values_match_if_adjacent_indices_match_and_next_access_is_a_read_operation;
    RAM_consistency_check_identity *= fake_alpha;
    RAM_consistency_check_identity += index_is_monotonically_increasing;
    RAM_consistency_check_identity *= fake_alpha;
    RAM_consistency_check_identity += next_gate_access_type_is_boolean;
    RAM_consistency_check_identity *= fake_alpha;
    RAM_consistency_check_identity += access_check;

    /**
     * RAM Timestamp Consistency Check
     */
    auto timestamp_delta = w_2_shift - w_2;
    auto RAM_timestamp_check_identity = (index_delta * FF(-1) + FF(1)) * timestamp_delta - w_3;

    /**
     * The complete RAM/ROM memory identity
     */
    auto memory_identity = ROM_consistency_check_identity * q_2;
    memory_identity += RAM_timestamp_check_identity * q_4;
    memory_identity += memory_record_check * q_m;
    memory_identity *= q_1;
    memory_identity += (RAM_consistency_check_identity * q_arith);

    auto expected_evals = memory_identity + non_native_field_identity + limb_accumulator_identity;
    expected_evals *= q_aux;

    validate_evaluations(expected_evals, relation, extended_edges, relation_parameters);
};

} // namespace proof_system::honk_relation_tests
