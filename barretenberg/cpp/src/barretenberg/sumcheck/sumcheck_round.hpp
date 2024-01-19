#pragma once
#include "barretenberg/common/thread.hpp"
#include "barretenberg/common/thread_utils.hpp"
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/polynomials/pow.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/relations/relation_types.hpp"
#include "barretenberg/relations/utils.hpp"

namespace bb::honk::sumcheck {

/*
 Notation: The polynomial P(X0, X1) that is the low-degree extension of its values vij = P(i,j)
 for i,j ∈ H = {0,1} is conveniently recorded as follows:
     (0,1)-----(1,1)   v01 ------ v11
       |          |     |          |  P(X0,X1) =   (v00 * (1-X0) + v10 * X0) * (1-X1)
   X1  |   H^2    |     | P(X0,X1) |             + (v01 * (1-X0) + v11 * X0) *   X1
       |          |     |          |
     (0,0) ---- (1,0)  v00 -------v10
            X0
*/

/*
 Example: There are two low-degree extensions Y1, Y2 over the square H^2 in the Cartesian plane.

    3 -------- 7   4 -------- 8
    |          |   |          | Let F(X0, X1) = G(Y1, Y2) =     G0(Y1(X0, X1), Y2(X0, X1))
    |    Y1    |   |    Y2    |                             + α G1(Y1(X0, X1), Y2(X0, X1)),
    |          |   |          |  where the relations are G0(Y1, Y2) = Y1 * Y2
    1 -------- 5   2 -------- 6                      and G1(Y1, Y2) = Y1 + Y2.

 G1, G2 together comprise the Relations.

 In the first round, the computations will relate elements along horizontal lines. As a mnemonic, we
 use the term "edge" for the linear, univariate polynomials corresponding to the four lines
  1 - 5
  2 - 6
  3 - 7
  4 - 8

 The polynomials Y1, Y2 are stored in an array in Multivariates. In the first round, these are arrays
 of spans living outside of the Multivariates object, and in sebsequent rounds these are arrays of field
 elements that are stored in the Multivariates. The rationale for adopting this model is to
 avoid copying the full-length polynomials; this way, the largest polynomial array stores in a
 Multivariates class is multivariates_n/2.

 Note: This class uses recursive function calls with template parameters. This is a common trick that is used to force
 the compiler to unroll loops. The idea is that a function that is only called once will always be inlined, and since
 template functions always create different functions, this is guaranteed.

 */

template <typename Flavor> class SumcheckProverRound {

    using Utils = bb::RelationUtils<Flavor>;
    using Relations = typename Flavor::Relations;
    using SumcheckTupleOfTuplesOfUnivariates = typename Flavor::SumcheckTupleOfTuplesOfUnivariates;
    using RelationSeparator = typename Flavor::RelationSeparator;

  public:
    using FF = typename Flavor::FF;
    using ExtendedEdges = typename Flavor::ExtendedEdges;

    size_t round_size; // a power of 2

    static constexpr size_t NUM_RELATIONS = Flavor::NUM_RELATIONS;
    static constexpr size_t MAX_PARTIAL_RELATION_LENGTH = Flavor::MAX_PARTIAL_RELATION_LENGTH;
    static constexpr size_t BATCHED_RELATION_PARTIAL_LENGTH = Flavor::BATCHED_RELATION_PARTIAL_LENGTH;

    SumcheckTupleOfTuplesOfUnivariates univariate_accumulators;

    // Prover constructor
    SumcheckProverRound(size_t initial_round_size)
        : round_size(initial_round_size)
    {
        // Initialize univariate accumulators to 0
        Utils::zero_univariates(univariate_accumulators);
    }

    /**
     * @brief Extend each edge in the edge group at to max-relation-length-many values.
     *
     * @details Should only be called externally with relation_idx equal to 0.
     * In practice, multivariates is one of ProverPolynomials or FoldedPolynomials.
     *
     */
    template <typename ProverPolynomialsOrPartiallyEvaluatedMultivariates>
    void extend_edges(ExtendedEdges& extended_edges,
                      const ProverPolynomialsOrPartiallyEvaluatedMultivariates& multivariates,
                      size_t edge_idx)
    {
        for (auto [extended_edge, multivariate] : zip_view(extended_edges.get_all(), multivariates.get_all())) {
            bb::Univariate<FF, 2> edge({ multivariate[edge_idx], multivariate[edge_idx + 1] });
            extended_edge = edge.template extend_to<MAX_PARTIAL_RELATION_LENGTH>();
        }
    }

    /**
     * @brief Return the evaluations of the univariate restriction (S_l(X_l) in the thesis) at num_multivariates-many
     * values. Most likely this will end up being S_l(0), ... , S_l(t-1) where t is around 12. At the end, reset all
     * univariate accumulators to be zero.
     */
    template <typename ProverPolynomialsOrPartiallyEvaluatedMultivariates>
    bb::Univariate<FF, BATCHED_RELATION_PARTIAL_LENGTH> compute_univariate(
        ProverPolynomialsOrPartiallyEvaluatedMultivariates& polynomials,
        const bb::RelationParameters<FF>& relation_parameters,
        const bb::PowPolynomial<FF>& pow_polynomial,
        const RelationSeparator alpha)
    {
        // Compute the constant contribution of pow polynomials for each edge. This is  the product of the partial
        // evaluation result c_l (i.e. pow(u_0,...,u_{l-1})) where u_0,...,u_{l-1} are the verifier challenges from
        // previous rounds) and the elements of pow(\vec{β}) not containing β_0,..., β_l.
        std::vector<FF> pow_challenges(round_size >> 1);
        pow_challenges[0] = pow_polynomial.partial_evaluation_result;
        for (size_t i = 1; i < (round_size >> 1); ++i) {
            pow_challenges[i] = pow_challenges[0] * pow_polynomial[i * pow_polynomial.periodicity];
        }

        // Determine number of threads for multithreading.
        // Note: Multithreading is "on" for every round but we reduce the number of threads from the max available based
        // on a specified minimum number of iterations per thread. This eventually leads to the use of a single thread.
        // For now we use a power of 2 number of threads simply to ensure the round size is evenly divided.
        size_t min_iterations_per_thread = 1 << 6; // min number of iterations for which we'll spin up a unique thread
        size_t num_threads = bb::thread_utils::calculate_num_threads_pow2(round_size, min_iterations_per_thread);
        size_t iterations_per_thread = round_size / num_threads; // actual iterations per thread

        // Construct univariate accumulator containers; one per thread
        std::vector<SumcheckTupleOfTuplesOfUnivariates> thread_univariate_accumulators(num_threads);
        for (auto& accum : thread_univariate_accumulators) {
            Utils::zero_univariates(accum);
        }

        // Construct extended edge containers; one per thread
        std::vector<ExtendedEdges> extended_edges;
        extended_edges.resize(num_threads);

        // Accumulate the contribution from each sub-relation accross each edge of the hyper-cube
        parallel_for(num_threads, [&](size_t thread_idx) {
            size_t start = thread_idx * iterations_per_thread;
            size_t end = (thread_idx + 1) * iterations_per_thread;

            for (size_t edge_idx = start; edge_idx < end; edge_idx += 2) {
                extend_edges(extended_edges[thread_idx], polynomials, edge_idx);

                // Compute the i-th edge's univariate contribution,
                // scale it by pow_challenge constant contribution and add it to the accumulators for Sˡ(Xₗ)
                accumulate_relation_univariates(thread_univariate_accumulators[thread_idx],
                                                extended_edges[thread_idx],
                                                relation_parameters,
                                                pow_challenges[edge_idx >> 1]);
            }
        });

        // Accumulate the per-thread univariate accumulators into a single set of accumulators
        for (auto& accumulators : thread_univariate_accumulators) {
            Utils::add_nested_tuples(univariate_accumulators, accumulators);
        }

        // Batch the univariate contributions from each sub-relation to obtain the round univariate
        return batch_over_relations<bb::Univariate<FF, BATCHED_RELATION_PARTIAL_LENGTH>>(
            univariate_accumulators, alpha, pow_polynomial);
    }

    /**
     * @brief Given a tuple t = (t_0, t_1, ..., t_{NUM_SUBRELATIONS-1}) and a challenge α,
     * return t_0 + αt_1 + ... + α^{NUM_SUBRELATIONS-1}t_{NUM_SUBRELATIONS-1}).
     */
    template <typename ExtendedUnivariate, typename ContainerOverSubrelations>
    static ExtendedUnivariate batch_over_relations(ContainerOverSubrelations& univariate_accumulators,
                                                   const RelationSeparator& challenge,
                                                   const bb::PowPolynomial<FF>& pow_polynomial)
    {
        auto running_challenge = FF(1);
        Utils::scale_univariates(univariate_accumulators, challenge, running_challenge);

        auto result = ExtendedUnivariate(0);
        extend_and_batch_univariates(univariate_accumulators, result, pow_polynomial);

        // Reset all univariate accumulators to 0 before beginning accumulation in the next round
        Utils::zero_univariates(univariate_accumulators);
        return result;
    }

    /**
     * @brief Extend Univariates to specified size then sum them
     *
     * @tparam extended_size Size after extension
     * @param tuple A tuple of tuples of Univariates
     * @param result A Univariate of length extended_size
     * @param pow_polynomial Power polynomial univariate
     */
    template <typename ExtendedUnivariate, typename TupleOfTuplesOfUnivariates>
    static void extend_and_batch_univariates(const TupleOfTuplesOfUnivariates& tuple,
                                             ExtendedUnivariate& result,
                                             const bb::PowPolynomial<FF>& pow_polynomial)
    {
        ExtendedUnivariate extended_random_polynomial;
        // Random poly R(X) = (1-X) + X.zeta_pow
        auto random_polynomial = bb::Univariate<FF, 2>({ 1, pow_polynomial.current_element() });
        extended_random_polynomial = random_polynomial.template extend_to<ExtendedUnivariate::LENGTH>();

        auto extend_and_sum = [&]<size_t relation_idx, size_t subrelation_idx, typename Element>(Element& element) {
            auto extended = element.template extend_to<ExtendedUnivariate::LENGTH>();

            using Relation = typename std::tuple_element_t<relation_idx, Relations>;
            const bool is_subrelation_linearly_independent =
                bb::subrelation_is_linearly_independent<Relation, subrelation_idx>();
            // Except from the log derivative subrelation, each other subrelation in part is required to be 0 hence we
            // multiply by the power polynomial. As the sumcheck prover is required to send a univariate to the
            // verifier, we additionally need a univariate contribution from the pow polynomial.
            if (!is_subrelation_linearly_independent) {
                result += extended;
            } else {
                result += extended * extended_random_polynomial;
            }
        };
        Utils::apply_to_tuple_of_tuples(tuple, extend_and_sum);
    }

  private:
    /**
     * @brief For a given edge, calculate the contribution of each relation to the prover round univariate (S_l in the
     * thesis).
     *
     * @details In Round l, the univariate S_l computed by the prover is computed as follows:
     *   - Outer loop: iterate through the points on the boolean hypercube of dimension = log(round_size), skipping
     *                 every other point. On each iteration, create a Univariate<FF, 2> (an 'edge') for each
     *                 multivariate.
     *   - Inner loop: iterate through the relations, feeding each relation the present collection of edges. Each
     *                 relation adds a contribution
     *
     * Result: for each relation, a univariate of some degree is computed by accumulating the contributions of each
     * group of edges. These are stored in `univariate_accumulators`. Adding these univariates together, with
     * appropriate scaling factors, produces S_l.
     */
    template <size_t relation_idx = 0>
    void accumulate_relation_univariates(SumcheckTupleOfTuplesOfUnivariates& univariate_accumulators,
                                         const auto& extended_edges,
                                         const bb::RelationParameters<FF>& relation_parameters,
                                         const FF& scaling_factor)
    {
        using Relation = std::tuple_element_t<relation_idx, Relations>;
        Relation::accumulate(
            std::get<relation_idx>(univariate_accumulators), extended_edges, relation_parameters, scaling_factor);

        // Repeat for the next relation.
        if constexpr (relation_idx + 1 < NUM_RELATIONS) {
            accumulate_relation_univariates<relation_idx + 1>(
                univariate_accumulators, extended_edges, relation_parameters, scaling_factor);
        }
    }
};

template <typename Flavor> class SumcheckVerifierRound {
    using Utils = bb::RelationUtils<Flavor>;
    using Relations = typename Flavor::Relations;
    using TupleOfArraysOfValues = typename Flavor::TupleOfArraysOfValues;
    using RelationSeparator = typename Flavor::RelationSeparator;

  public:
    using FF = typename Flavor::FF;
    using ClaimedEvaluations = typename Flavor::AllValues;

    bool round_failed = false;

    static constexpr size_t NUM_RELATIONS = Flavor::NUM_RELATIONS;
    static constexpr size_t BATCHED_RELATION_PARTIAL_LENGTH = Flavor::BATCHED_RELATION_PARTIAL_LENGTH;

    FF target_total_sum = 0;

    TupleOfArraysOfValues relation_evaluations;

    // Verifier constructor
    explicit SumcheckVerifierRound(FF target_total_sum = 0)
        : target_total_sum(target_total_sum)
    {
        Utils::zero_elements(relation_evaluations);
    };

    bool check_sum(bb::Univariate<FF, BATCHED_RELATION_PARTIAL_LENGTH>& univariate)
    {
        // S^{l}(0) = ( (1−0) + 0⋅ζ^{ 2^l } ) ⋅ T^{l}(0) = T^{l}(0)
        // S^{l}(1) = ( (1−1) + 1⋅ζ^{ 2^l } ) ⋅ T^{l}(1) = ζ^{ 2^l } ⋅ T^{l}(1)
        FF total_sum = univariate.value_at(0) + univariate.value_at(1);
        // target_total_sum = sigma_{l} =
        // TODO(#673): Conditionals like this can go away once native verification is is just recursive verification
        // with a simulated builder.
        bool sumcheck_round_failed(false);
        if constexpr (IsRecursiveFlavor<Flavor>) {
            target_total_sum.assert_equal(total_sum);
            sumcheck_round_failed = (target_total_sum != total_sum).get_value();
        } else {
            sumcheck_round_failed = (target_total_sum != total_sum);
        }

        round_failed = round_failed || sumcheck_round_failed;
        return !sumcheck_round_failed;
    };

    /**
     * @brief After checking that the univariate is good for this round, compute the next target sum.
     *
     * @param univariate T^l(X), given by its evaluations over {0,1,2,...},
     * equal to S^{l}(X)/( (1−X) + X⋅ζ^{ 2^l } )
     * @param round_challenge u_l
     * @return FF sigma_{l+1} = S^l(u_l)
     */
    FF compute_next_target_sum(bb::Univariate<FF, BATCHED_RELATION_PARTIAL_LENGTH>& univariate, FF& round_challenge)
    {
        // Evaluate T^{l}(u_{l})
        target_total_sum = univariate.evaluate(round_challenge);
        return target_total_sum;
    }

    /**
     * @brief General purpose method for applying a tuple of arrays (of FFs)
     *
     * @tparam Operation Any operation valid on elements of the inner arrays (FFs)
     * @param tuple Tuple of arrays (of FFs)
     */
    // also copy paste in PG
    // so instead of having claimed evaluations of each relation in part  you have the actual evaluations
    // kill the pow_univariat
    FF compute_full_honk_relation_purported_value(ClaimedEvaluations purported_evaluations,
                                                  const bb::RelationParameters<FF>& relation_parameters,
                                                  const bb::PowPolynomial<FF>& pow_polynomial,
                                                  const RelationSeparator alpha)
    {
        Utils::template accumulate_relation_evaluations<>(
            purported_evaluations, relation_evaluations, relation_parameters, pow_polynomial.partial_evaluation_result);

        auto running_challenge = FF(1);
        auto output = FF(0);
        Utils::scale_and_batch_elements(relation_evaluations, alpha, running_challenge, output);
        return output;
    }
};
} // namespace bb::honk::sumcheck
