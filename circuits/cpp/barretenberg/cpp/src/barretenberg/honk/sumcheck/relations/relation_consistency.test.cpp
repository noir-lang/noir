#include "../polynomials/barycentric_data.hpp"
#include "../polynomials/univariate.hpp"
#include "arithmetic_relation.hpp"
#include "barretenberg/honk/flavor/standard.hpp"
#include "barretenberg/honk/sumcheck/relations/lookup_relation.hpp"
#include "barretenberg/honk/sumcheck/relations/ultra_arithmetic_relation.hpp"
#include "permutation_relation.hpp"
#include "relation_parameters.hpp"

#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/numeric/random/engine.hpp"

#include <cstddef>
#include <gtest/gtest.h>
using namespace proof_system::honk::sumcheck;
/**
 * The purpose of this test suite is to show that the identity arithmetic implemented in the Relations is equivalent to
 * a simpler unoptimized version implemented in the tests themselves. This is useful 1) as documentation since the
 * simple implementations here should make the underlying arithmetic easier to see, and 2) as a check that optimizations
 * introduced into the Relations have not changed the result.
 *
 * For this purpose, we simply feed (the same) random inputs into each of the two implementations and confirm that
 * the outputs match. This does not confirm the correctness of the identity arithmetic (the identities will not be
 * satisfied in general by random inputs) only that the two implementations are equivalent.
 */
static const size_t INPUT_UNIVARIATE_LENGTH = 2;

namespace proof_system::honk_relation_tests {

class StandardRelationConsistency : public testing::Test {
  public:
    using Flavor = honk::flavor::Standard;
    using FF = typename Flavor::FF;
    using ClaimedEvaluations = typename Flavor::ClaimedEvaluations;
    // TODO(#390): Move MAX_RELATION_LENGTH into Flavor and simplify this.

    template <size_t t> using ExtendedEdges = typename Flavor::template ExtendedEdges<t>;

    // TODO(#225)(Adrian): Accept FULL_RELATION_LENGTH as a template parameter for this function only, so that the
    // test can decide to which degree the polynomials must be extended. Possible accept an existing list of
    // "edges" and extend them to the degree.
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
    static void validate_evaluations(const auto& expected_full_length_univariates, /* array of Univariates*/
                                     const auto relation,
                                     const ExtendedEdges<FULL_RELATION_LENGTH>& extended_edges,
                                     const RelationParameters<FF>& relation_parameters)
    {
        // First check that the verifier's computation on individual evaluations is correct.
        // Note: since add_full_relation_value_contribution computes the identities at a single evaluation of the
        // multivariates, we need only pass in one evaluation point from the extended edges. Which one we choose is
        // arbitrary so we choose the 0th.

        // Extract the RelationValues type for the given relation
        using RelationValues = typename decltype(relation)::RelationValues;
        RelationValues relation_evals;
        RelationValues expected_relation_evals;

        ASSERT_EQ(expected_relation_evals.size(), expected_full_length_univariates.size());
        // Initialize expected_evals to 0th coefficient of expected full length univariates
        for (size_t idx = 0; idx < relation_evals.size(); ++idx) {
            relation_evals[idx] = FF(0); // initialize to 0
            expected_relation_evals[idx] = expected_full_length_univariates[idx].value_at(0);
        }

        // Extract 0th evaluation from extended edges
        ClaimedEvaluations edge_evaluations = transposed_univariate_array_at(extended_edges, 0);

        // Evaluate the relation using the verifier functionality
        relation.add_full_relation_value_contribution(relation_evals, edge_evaluations, relation_parameters);

        EXPECT_EQ(relation_evals, expected_relation_evals);

        // Next, check that the prover's computation on Univariates is correct

        using RelationUnivariates = typename decltype(relation)::RelationUnivariates;
        RelationUnivariates relation_univariates;
        zero_univariates<>(relation_univariates);

        constexpr std::size_t num_univariates = std::tuple_size<RelationUnivariates>::value;

        // Compute the relatiion univariates via the sumcheck prover functionality, then extend
        // them to full length for easy comparison with the expected result.
        relation.add_edge_contribution(relation_univariates, extended_edges, relation_parameters, 1);

        auto full_length_univariates = std::array<Univariate<FF, FULL_RELATION_LENGTH>, num_univariates>();
        extend_tuple_of_arrays<FULL_RELATION_LENGTH>(relation_univariates, full_length_univariates);

        EXPECT_EQ(full_length_univariates, expected_full_length_univariates);
    };

    template <size_t idx = 0, typename... Ts> static void zero_univariates(std::tuple<Ts...>& tuple)
    {
        auto& element = std::get<idx>(tuple);
        std::fill(element.evaluations.begin(), element.evaluations.end(), FF(0));

        if constexpr (idx + 1 < sizeof...(Ts)) {
            zero_univariates<idx + 1>(tuple);
        }
    }

    template <size_t extended_size, size_t idx = 0, typename... Ts>
    static void extend_tuple_of_arrays(std::tuple<Ts...>& tuple, auto& result_univariates)
    {
        auto& element = std::get<idx>(tuple);
        using Element = std::remove_reference_t<decltype(element)>;
        BarycentricData<FF, Element::LENGTH, extended_size> barycentric_utils;
        result_univariates[idx] = barycentric_utils.extend(element);

        if constexpr (idx + 1 < sizeof...(Ts)) {
            extend_tuple_of_arrays<extended_size, idx + 1>(tuple, result_univariates);
        }
    }
};

TEST_F(StandardRelationConsistency, ArithmeticRelation)
{
    using Flavor = honk::flavor::Standard;
    using FF = typename Flavor::FF;
    static constexpr size_t FULL_RELATION_LENGTH = 5;
    using ExtendedEdges = typename Flavor::template ExtendedEdges<FULL_RELATION_LENGTH>;
    static const size_t NUM_POLYNOMIALS = Flavor::NUM_ALL_ENTITIES;

    const auto relation_parameters = compute_mock_relation_parameters();
    auto run_test = [&relation_parameters](bool is_random_input) {
        std::array<Univariate<FF, INPUT_UNIVARIATE_LENGTH>, NUM_POLYNOMIALS> input_polynomials;
        ExtendedEdges extended_edges;
        if (!is_random_input) {
            // evaluation form, i.e. input_univariate(0) = 1, input_univariate(1) = 2,.. The polynomial is x+1.
            for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
                input_polynomials[i] = Univariate<FF, INPUT_UNIVARIATE_LENGTH>({ 1, 2 });
            }
            compute_mock_extended_edges<FULL_RELATION_LENGTH>(extended_edges, input_polynomials);
        } else {
            // input_univariates are random polynomials of degree one
            for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
                input_polynomials[i] =
                    Univariate<FF, INPUT_UNIVARIATE_LENGTH>({ FF::random_element(), FF::random_element() });
            }
            compute_mock_extended_edges<FULL_RELATION_LENGTH>(extended_edges, input_polynomials);
        };
        auto relation = ArithmeticRelation<FF>();
        // Manually compute the expected edge contribution
        const auto& w_l = extended_edges.w_l;
        const auto& w_r = extended_edges.w_r;
        const auto& w_o = extended_edges.w_o;
        const auto& q_m = extended_edges.q_m;
        const auto& q_l = extended_edges.q_l;
        const auto& q_r = extended_edges.q_r;
        const auto& q_o = extended_edges.q_o;
        const auto& q_c = extended_edges.q_c;

        // Compute expected full length Univariates using straight forward expressions.
        // Note: expect { { 5, 22, 57, 116, 205} } for input polynomial {1, 2}
        constexpr std::size_t NUM_SUBRELATIONS = std::tuple_size_v<decltype(relation)::RelationUnivariates>;
        auto expected_full_length_univariates = std::array<Univariate<FF, FULL_RELATION_LENGTH>, NUM_SUBRELATIONS>();

        expected_full_length_univariates[0] = (q_m * w_r * w_l) + (q_r * w_r) + (q_l * w_l) + (q_o * w_o) + (q_c);
        validate_evaluations(expected_full_length_univariates, relation, extended_edges, relation_parameters);
    };
    run_test(/* is_random_input=*/true);
    run_test(/* is_random_input=*/false);
};

TEST_F(StandardRelationConsistency, PermutationRelation)
{
    using Flavor = honk::flavor::Standard;
    using FF = typename Flavor::FF;
    static constexpr size_t FULL_RELATION_LENGTH = 5;
    using ExtendedEdges = typename Flavor::template ExtendedEdges<FULL_RELATION_LENGTH>;
    static const size_t NUM_POLYNOMIALS = Flavor::NUM_ALL_ENTITIES;

    const auto relation_parameters = compute_mock_relation_parameters();
    auto run_test = [&relation_parameters](bool is_random_input) {
        ExtendedEdges extended_edges;
        std::array<Univariate<FF, INPUT_UNIVARIATE_LENGTH>, NUM_POLYNOMIALS> input_polynomials;
        if (!is_random_input) {
            // evaluation form, i.e. input_univariate(0) = 1, input_univariate(1) = 2,.. The polynomial is x+1.
            for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
                input_polynomials[i] = Univariate<FF, INPUT_UNIVARIATE_LENGTH>({ 1, 2 });
            }
            compute_mock_extended_edges<FULL_RELATION_LENGTH>(extended_edges, input_polynomials);
        } else {
            // input_univariates are random polynomials of degree one
            for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
                input_polynomials[i] =
                    Univariate<FF, INPUT_UNIVARIATE_LENGTH>({ FF::random_element(), FF::random_element() });
            }
            compute_mock_extended_edges<FULL_RELATION_LENGTH>(extended_edges, input_polynomials);
        };
        auto relation = PermutationRelation<FF>();

        const auto& beta = relation_parameters.beta;
        const auto& gamma = relation_parameters.gamma;
        const auto& public_input_delta = relation_parameters.public_input_delta;

        // Manually compute the expected edge contribution
        const auto& w_1 = extended_edges.w_l;
        const auto& w_2 = extended_edges.w_r;
        const auto& w_3 = extended_edges.w_o;
        const auto& sigma_1 = extended_edges.sigma_1;
        const auto& sigma_2 = extended_edges.sigma_2;
        const auto& sigma_3 = extended_edges.sigma_3;
        const auto& id_1 = extended_edges.id_1;
        const auto& id_2 = extended_edges.id_2;
        const auto& id_3 = extended_edges.id_3;
        const auto& z_perm = extended_edges.z_perm;
        const auto& z_perm_shift = extended_edges.z_perm_shift;
        const auto& lagrange_first = extended_edges.lagrange_first;
        const auto& lagrange_last = extended_edges.lagrange_last;

        // Compute expected full length Univariates using straight forward expressions
        constexpr std::size_t NUM_SUBRELATIONS = std::tuple_size_v<decltype(relation)::RelationUnivariates>;
        auto expected_full_length_univariates = std::array<Univariate<FF, FULL_RELATION_LENGTH>, NUM_SUBRELATIONS>();

        expected_full_length_univariates[0] = (z_perm + lagrange_first) * (w_1 + id_1 * beta + gamma) *
                                                  (w_2 + id_2 * beta + gamma) * (w_3 + id_3 * beta + gamma) -
                                              (z_perm_shift + lagrange_last * public_input_delta) *
                                                  (w_1 + sigma_1 * beta + gamma) * (w_2 + sigma_2 * beta + gamma) *
                                                  (w_3 + sigma_3 * beta + gamma);

        expected_full_length_univariates[1] = z_perm_shift * lagrange_last;

        validate_evaluations(expected_full_length_univariates, relation, extended_edges, relation_parameters);
    };
    run_test(/* is_random_input=*/true);
    run_test(/* is_random_input=*/false);
};

} // namespace proof_system::honk_relation_tests
