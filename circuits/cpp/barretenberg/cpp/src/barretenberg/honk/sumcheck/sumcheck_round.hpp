#pragma once
#include "barretenberg/common/log.hpp"
#include <array>
#include <algorithm>
#include <tuple>
#include "polynomials/barycentric_data.hpp"
#include "polynomials/univariate.hpp"
#include "polynomials/pow.hpp"
#include "relations/relation_parameters.hpp"
#include "barretenberg/honk/flavor/ultra.hpp"
#include <functional>

namespace proof_system::honk::sumcheck {

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

 @todo TODO(#390): Template only on Flavor? Is it useful to have these decoupled?
 */

template <typename Flavor> class SumcheckRound {

    using Relations = typename Flavor::Relations;
    using RelationUnivariates = typename Flavor::RelationUnivariates;
    using RelationEvaluations = typename Flavor::RelationValues;

  public:
    using FF = typename Flavor::FF;
    template <size_t univariate_length>
    using ExtendedEdges = typename Flavor::template ExtendedEdges<univariate_length>;
    using ClaimedEvaluations = typename Flavor::ClaimedEvaluations;

    bool round_failed = false;
    size_t round_size; // a power of 2

    Relations relations;
    static constexpr size_t NUM_RELATIONS = Flavor::NUM_RELATIONS;
    static constexpr size_t MAX_RELATION_LENGTH = Flavor::MAX_RELATION_LENGTH;

    FF target_total_sum = 0;

    RelationUnivariates univariate_accumulators;
    RelationEvaluations relation_evaluations;

    ExtendedEdges<MAX_RELATION_LENGTH> extended_edges;

    // TODO(#224)(Cody): this should go away
    BarycentricData<FF, 2, MAX_RELATION_LENGTH> barycentric_2_to_max = BarycentricData<FF, 2, MAX_RELATION_LENGTH>();

    // Prover constructor
    SumcheckRound(size_t initial_round_size)
        : round_size(initial_round_size)
    {
        // Initialize univariate accumulators to 0
        zero_univariates(univariate_accumulators);
    }

    // Verifier constructor
    explicit SumcheckRound() { zero_elements(relation_evaluations); };

    /**
     * @brief Given a tuple t = (t_0, t_1, ..., t_{NUM_RELATIONS-1}) and a challenge α,
     * return t_0 + αt_1 + ... + α^{NUM_RELATIONS-1}t_{NUM_RELATIONS-1}).
     *
     * @tparam T : In practice, this is a Univariate<FF, MAX_NUM_RELATIONS>.
     */
    Univariate<FF, MAX_RELATION_LENGTH> batch_over_relations(FF challenge)
    {
        FF running_challenge = 1;
        scale_univariates(univariate_accumulators, challenge, running_challenge);

        auto result = Univariate<FF, MAX_RELATION_LENGTH>();
        extend_and_batch_univariates(univariate_accumulators, result);

        // Reset all univariate accumulators to 0 before beginning accumulation in the next round
        zero_univariates(univariate_accumulators);
        return result;
    }

    /**
     * @brief Extend each edge in the edge group at to max-relation-length-many values.
     *
     * @details Should only be called externally with relation_idx equal to 0.
     * In practice, multivariates is one of ProverPolynomials or FoldedPolynomials.
     *
     */
    void extend_edges(auto& multivariates, size_t edge_idx)
    {
        size_t univariate_idx = 0; // TODO(#391) zip
        for (auto& poly : multivariates) {
            auto edge = Univariate<FF, 2>({ poly[edge_idx], poly[edge_idx + 1] });
            extended_edges[univariate_idx] = barycentric_2_to_max.extend(edge);
            ++univariate_idx;
        }
    }

    /**
     * @brief Return the evaluations of the univariate restriction (S_l(X_l) in the thesis) at num_multivariates-many
     * values. Most likely this will end up being S_l(0), ... , S_l(t-1) where t is around 12. At the end, reset all
     * univariate accumulators to be zero.
     */
    Univariate<FF, MAX_RELATION_LENGTH> compute_univariate(auto& polynomials,
                                                           const RelationParameters<FF>& relation_parameters,
                                                           const PowUnivariate<FF>& pow_univariate,
                                                           const FF alpha)
    {
        // For each edge_idx = 2i, we need to multiply the whole contribution by zeta^{2^{2i}}
        // This means that each univariate for each relation needs an extra multiplication.
        FF pow_challenge = pow_univariate.partial_evaluation_constant;
        for (size_t edge_idx = 0; edge_idx < round_size; edge_idx += 2) {
            extend_edges(polynomials, edge_idx);

            // Compute the i-th edge's univariate contribution,
            // scale it by the pow polynomial's constant and zeta power "c_l ⋅ ζ_{l+1}ⁱ"
            // and add it to the accumulators for Sˡ(Xₗ)
            accumulate_relation_univariates<>(relation_parameters, pow_challenge);
            // Update the pow polynomial's contribution c_l ⋅ ζ_{l+1}ⁱ for the next edge.
            pow_challenge *= pow_univariate.zeta_pow_sqr;
        }

        return batch_over_relations(alpha);
    }

    /**
     * @brief Calculate the contribution of each relation to the expected value of the full Honk relation.
     *
     * @details For each relation, use the purported values (supplied by the prover) of the multivariates to calculate
     * a contribution to the purported value of the full Honk relation. These are stored in `evaluations`. Adding these
     * together, with appropriate scaling factors, produces the expected value of the full Honk relation. This value is
     * checked against the final value of the target total sum, defined as sigma_d.
     */
    FF compute_full_honk_relation_purported_value(ClaimedEvaluations purported_evaluations,
                                                  const RelationParameters<FF>& relation_parameters,
                                                  const PowUnivariate<FF>& pow_univariate,
                                                  const FF alpha)
    {
        accumulate_relation_evaluations<>(purported_evaluations, relation_parameters);

        auto running_challenge = FF(1);
        auto output = FF(0);
        scale_and_batch_elements(relation_evaluations, alpha, running_challenge, output);

        output *= pow_univariate.partial_evaluation_constant;

        return output;
    }

    /**
     * @brief check if S^{l}(0) + S^{l}(1) = S^{l-1}(u_{l-1}) = sigma_{l} (or 0 if l=0)
     *
     * @param univariate T^{l}(X), the round univariate that is equal to S^{l}(X)/( (1−X) + X⋅ζ^{ 2^l } )
     */
    bool check_sum(Univariate<FF, MAX_RELATION_LENGTH>& univariate, const PowUnivariate<FF>& pow_univariate)
    {
        // S^{l}(0) = ( (1−0) + 0⋅ζ^{ 2^l } ) ⋅ T^{l}(0) = T^{l}(0)
        // S^{l}(1) = ( (1−1) + 1⋅ζ^{ 2^l } ) ⋅ T^{l}(1) = ζ^{ 2^l } ⋅ T^{l}(1)
        FF total_sum = univariate.value_at(0) + (pow_univariate.zeta_pow * univariate.value_at(1));
        // target_total_sum = sigma_{l} =
        bool sumcheck_round_failed = (target_total_sum != total_sum);
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
    FF compute_next_target_sum(Univariate<FF, MAX_RELATION_LENGTH>& univariate,
                               FF& round_challenge,
                               const PowUnivariate<FF>& pow_univariate)
    {
        // IMPROVEMENT(Cody): Use barycentric static method, maybe implement evaluation as member
        // function on Univariate.
        auto barycentric = BarycentricData<FF, MAX_RELATION_LENGTH, MAX_RELATION_LENGTH>();
        // Evaluate T^{l}(u_{l})
        target_total_sum = barycentric.evaluate(univariate, round_challenge);
        // Evaluate (1−u_l) + u_l ⋅ ζ^{2^l} )
        FF pow_monomial_eval = pow_univariate.univariate_eval(round_challenge);
        // sigma_{l+1} = S^l(u_l) = (1−u_l) + u_l⋅ζ^{2^l} ) ⋅ T^{l}(u_l)
        target_total_sum *= pow_monomial_eval;
        return target_total_sum;
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
    void accumulate_relation_univariates(const RelationParameters<FF>& relation_parameters, const FF& scaling_factor)
    {
        std::get<relation_idx>(relations).add_edge_contribution(
            std::get<relation_idx>(univariate_accumulators), extended_edges, relation_parameters, scaling_factor);

        // Repeat for the next relation.
        if constexpr (relation_idx + 1 < NUM_RELATIONS) {
            accumulate_relation_univariates<relation_idx + 1>(relation_parameters, scaling_factor);
        }
    }

    // TODO(#224)(Cody): make uniform with accumulate_relation_univariates
    /**
     * @brief Calculate the contribution of each relation to the expected value of the full Honk relation.
     *
     * @details For each relation, use the purported values (supplied by the prover) of the multivariates to calculate
     * a contribution to the purported value of the full Honk relation. These are stored in `evaluations`. Adding these
     * together, with appropriate scaling factors, produces the expected value of the full Honk relation. This value is
     * checked against the final value of the target total sum (called sigma_0 in the thesis).
     */
    template <size_t relation_idx = 0>
    // TODO(#224)(Cody): Input should be an array?
    void accumulate_relation_evaluations(ClaimedEvaluations purported_evaluations,
                                         const RelationParameters<FF>& relation_parameters)
    {
        std::get<relation_idx>(relations).add_full_relation_value_contribution(
            std::get<relation_idx>(relation_evaluations), purported_evaluations, relation_parameters);

        // Repeat for the next relation.
        if constexpr (relation_idx + 1 < NUM_RELATIONS) {
            accumulate_relation_evaluations<relation_idx + 1>(purported_evaluations, relation_parameters);
        }
    }

  public:
    /**
     * Utility methods for tuple of tuples of Univariates
     */

    /**
     * @brief Extend Univariates to specified size then sum them
     *
     * @tparam extended_size Size after extension
     * @param tuple A tuple of tuples of Univariates
     * @param result A Univariate of length extended_size
     */
    template <size_t extended_size>
    static void extend_and_batch_univariates(auto& tuple, Univariate<FF, extended_size>& result)
    {
        auto extend_and_sum = [&](auto& element) {
            using Element = std::remove_reference_t<decltype(element)>;
            // TODO(#224)(Cody): this barycentric stuff should be more built-in?
            BarycentricData<FF, Element::LENGTH, extended_size> barycentric_utils;
            result += barycentric_utils.extend(element);
        };
        apply_to_tuple_of_tuples(tuple, extend_and_sum);
    }

    /**
     * @brief Set all coefficients of Univariates to zero
     *
     * @details After computing the round univariate, it is necessary to zero-out the accumulators used to compute it.
     */
    static void zero_univariates(auto& tuple)
    {
        auto set_to_zero = [](auto& element) {
            std::fill(element.evaluations.begin(), element.evaluations.end(), FF(0));
        };
        apply_to_tuple_of_tuples(tuple, set_to_zero);
    }

    /**
     * @brief Scale Univaraites by consecutive powers of the provided challenge
     *
     * @param tuple Tuple of tuples of Univariates
     * @param challenge
     * @param current_scalar power of the challenge
     */
    static void scale_univariates(auto& tuple, const FF& challenge, FF current_scalar)
    {
        auto scale_by_consecutive_powers_of_challenge = [&](auto& element) {
            element *= current_scalar;
            current_scalar *= challenge;
        };
        apply_to_tuple_of_tuples(tuple, scale_by_consecutive_powers_of_challenge);
    }

    /**
     * @brief General purpose method for applying an operation to a tuple of tuples of Univariates
     *
     * @tparam Operation Any operation valid on Univariates
     * @tparam outer_idx Index into the outer tuple
     * @tparam inner_idx Index into the inner tuple
     * @param tuple A Tuple of tuples of Univariates
     * @param operation Operation to apply to Univariates
     */
    template <typename Operation, size_t outer_idx = 0, size_t inner_idx = 0>
    static void apply_to_tuple_of_tuples(auto& tuple, Operation&& operation)
    {
        auto& inner_tuple = std::get<outer_idx>(tuple);
        auto& univariate = std::get<inner_idx>(inner_tuple);

        // Apply the specified operation to each Univariate
        std::invoke(std::forward<Operation>(operation), univariate);

        const size_t inner_size = std::tuple_size_v<std::decay_t<decltype(std::get<outer_idx>(tuple))>>;
        const size_t outer_size = std::tuple_size_v<std::decay_t<decltype(tuple)>>;

        // Recurse over inner and outer tuples
        if constexpr (inner_idx + 1 < inner_size) {
            apply_to_tuple_of_tuples<Operation, outer_idx, inner_idx + 1>(tuple, std::forward<Operation>(operation));
        } else if constexpr (outer_idx + 1 < outer_size) {
            apply_to_tuple_of_tuples<Operation, outer_idx + 1, 0>(tuple, std::forward<Operation>(operation));
        }
    }

    /**
     * Utility methods for tuple of arrays
     */

    /**
     * @brief Set each element in a tuple of arrays to zero.
     * @details FF's default constructor may not initialize to zero (e.g., barretenberg::fr), hence we can't rely on
     * aggregate initialization of the evaluations array.
     */
    template <size_t idx = 0> static void zero_elements(auto& tuple)
    {
        auto set_to_zero = [](auto& element) { std::fill(element.begin(), element.end(), FF(0)); };
        apply_to_tuple_of_arrays(set_to_zero, tuple);
    };

    /**
     * @brief Scale elements by consecutive powers of the challenge then sum
     * @param result Batched result
     */
    static void scale_and_batch_elements(auto& tuple, const FF& challenge, FF current_scalar, FF& result)
    {
        auto scale_by_challenge_and_accumulate = [&](auto& element) {
            for (auto& entry : element) {
                result += entry * current_scalar;
                current_scalar *= challenge;
            }
        };
        apply_to_tuple_of_arrays(scale_by_challenge_and_accumulate, tuple);
    }

    /**
     * @brief General purpose method for applying a tuple of arrays (of FFs)
     *
     * @tparam Operation Any operation valid on elements of the inner arrays (FFs)
     * @param tuple Tuple of arrays (of FFs)
     */
    template <typename Operation, size_t idx = 0, typename... Ts>
    static void apply_to_tuple_of_arrays(Operation&& operation, std::tuple<Ts...>& tuple)
    {
        auto& element = std::get<idx>(tuple);

        std::invoke(std::forward<Operation>(operation), element);

        if constexpr (idx + 1 < sizeof...(Ts)) {
            apply_to_tuple_of_arrays<Operation, idx + 1>(operation, tuple);
        }
    }
};
} // namespace proof_system::honk::sumcheck
