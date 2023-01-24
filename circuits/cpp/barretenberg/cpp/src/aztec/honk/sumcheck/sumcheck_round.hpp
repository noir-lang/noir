#include <common/log.hpp>
#include <array>
#include <algorithm>
#include <tuple>
#include "polynomials/barycentric_data.hpp"
#include "polynomials/univariate.hpp"
namespace honk::sumcheck {

/*
 Notation: The polynomial P(X1, X2) that is the low-degree extension of its values vij = P(i,j)
 for i,j ∈ H = {0,1} is conveniently recorded as follows:
     (0,1)-----(1,1)   v01 ------ v11
       |          |     |          |  P(X1,X2) =   (v01 * (1-X1) + v11 * X1) *   X2
       |   H^2    |     | P(X1,X2) |             + (v00 * (1-X1) + v10 * X1) * (1-X2)
       |          |     |          |
     (0,0) ---- (1,0)  v00 -------v10
*/

/*
 Example: There are two low-degree extensions Y1, Y2 over the square H^2 in the Cartesian plane.

    3 -------- 7   4 -------- 8
    |          |   |          | Let F(X1, X2) = G(Y1, Y2) =     G0(Y1(X1, X2) + Y2(X1, X2))
    |    Y1    |   |    Y2    |                             + α G1(Y1(X1, X2) + Y2(X1, X2)),
    |          |   |          |  where the relations are G0(Y1, Y2) = Y1 * Y2
    1 -------- 5   2 -------- 6                      and G1(Y1, Y2) = Y1 + Y2.

 G1, G2 together comprise the Relations.

 In the first round, the computations will relate elements along veritcal lines. As a mnemonic, we
 use the term "edge" for the linear, univariate polynomials corresponding to the four lines
  3   4   7   8
  | , | , | , | .
  1   2   5   6

 The polynomials Y1, Y2 are stored in an array in Multivariates. In the first round, these are arrays
 of spans living outside of the Multivariates object, and in sebsequent rounts these are arrays of field
 elements that are stored in the Multivariates. The rationale for adopting this model is to
 avoid copying the full-length polynomials; this way, the largest polynomial array stores in a
 Multivariates class is multivariates_n/2.
 */

template <class FF, size_t num_multivariates, template <class> class... Relations> class SumcheckRound {

  public:
    bool round_failed = false;
    size_t round_size; // a power of 2

    std::tuple<Relations<FF>...> relations;
    static constexpr size_t NUM_RELATIONS = sizeof...(Relations);
    static constexpr size_t MAX_RELATION_LENGTH = std::max({ Relations<FF>::RELATION_LENGTH... });

    FF target_total_sum = 0;

    // TODO(Cody): this barycentric stuff should be more built-in?
    std::tuple<BarycentricData<FF, Relations<FF>::RELATION_LENGTH, MAX_RELATION_LENGTH>...> barycentric_utils;
    std::tuple<Univariate<FF, Relations<FF>::RELATION_LENGTH>...> univariate_accumulators;
    std::array<FF, NUM_RELATIONS> evaluations;
    std::array<Univariate<FF, MAX_RELATION_LENGTH>, num_multivariates> extended_edges;
    std::array<Univariate<FF, MAX_RELATION_LENGTH>, num_multivariates> extended_univariates;

    // TODO(Cody): this should go away and we should use constexpr method to extend
    BarycentricData<FF, 2, MAX_RELATION_LENGTH> barycentric_2_to_max = BarycentricData<FF, 2, MAX_RELATION_LENGTH>();

    // Prover constructor
    SumcheckRound(size_t initial_round_size, auto&& relations)
        : round_size(initial_round_size)
        , relations(relations)
        , barycentric_utils(BarycentricData<FF, Relations<FF>::RELATION_LENGTH, MAX_RELATION_LENGTH>()...)
        , univariate_accumulators(Univariate<FF, Relations<FF>::RELATION_LENGTH>()...)
    {}

    // Verifier constructor
    explicit SumcheckRound(auto relations)
        : relations(relations)
        // TODO(Cody): this is a hack; accumulators not needed by verifier
        , univariate_accumulators(Univariate<FF, Relations<FF>::RELATION_LENGTH>()...)
    {
        // FF's default constructor may not initialize to zero (e.g., barretenberg::fr), hence we can't rely on
        // aggregate initialization of the evaluations array.
        std::fill(evaluations.begin(), evaluations.end(), FF(0));
    };

    /**
     * @brief After computing the round univariate, it is necessary to zero-out the accumulators used to compute it.
     */
    template <size_t idx = 0> void reset_accumulators()
    {
        auto& univariate = std::get<idx>(univariate_accumulators);
        std::fill(univariate.evaluations.begin(), univariate.evaluations.end(), FF(0));

        if constexpr (idx + 1 < NUM_RELATIONS) {
            reset_accumulators<idx + 1>();
        }
    };
    // IMPROVEMENT(Cody): This is kind of ugly. There should be a one-liner with folding
    // or std::apply or something.

    /**
     * @brief Given a tuple t = (t_0, t_1, ..., t_{NUM_RELATIONS-1}) and a challenge α,
     * modify the tuple in place to (t_0, αt_1, ..., α^{NUM_RELATIONS-1}t_{NUM_RELATIONS-1}).
     */
    template <size_t idx = 0> void scale_tuple(auto& tuple, FF challenge, FF running_challenge)
    {
        std::get<idx>(tuple) *= running_challenge;
        running_challenge *= challenge;
        if constexpr (idx + 1 < NUM_RELATIONS) {
            scale_tuple<idx + 1>(tuple, challenge, running_challenge);
        }
    };

    /**
     * @brief Given a tuple t = (t_0, t_1, ..., t_{NUM_RELATIONS-1}) and a challenge α,
     * return t_0 + αt_1 + ... + α^{NUM_RELATIONS-1}t_{NUM_RELATIONS-1}).
     *
     * @tparam T : In practice, this is an FF or a Univariate<FF, MAX_NUM_RELATIONS>.
     */
    template <typename T> T batch_over_relations(auto& tuple, FF challenge)
    {
        FF running_challenge = 1;
        scale_tuple<>(tuple, challenge, running_challenge);
        extend_univariate_accumulators<>();
        auto result = T();
        for (size_t i = 0; i < NUM_RELATIONS; ++i) {
            result += extended_univariates[i];
        }
        return result;
    }

    /**
     * @brief Evaluate some relations by evaluating each edge in the edge group at
     * Univariate::length-many values. Store each value separately in the corresponding
     * entry of relation_evals.
     *
     * @details Should only be called externally with relation_idx equal to 0.
     *
     */
    template <typename T> void extend_edges(T multivariate, size_t edge_idx)
    {
        for (size_t idx = 0; idx < num_multivariates; idx++) {
            auto edge = Univariate<FF, 2>({ multivariate[idx][edge_idx], multivariate[idx][edge_idx + 1] });
            extended_edges[idx] = barycentric_2_to_max.extend(edge);
        }
    }

    // TODO(Cody): make private
    /**
     * @brief For a given edge, calculate the contribution of each relation to the prover round univariate (S_l in the
     * thesis).
     *
     * @details In Round l, the univariate S_l computed by the prover is computed as follows:
     *   - Outer loop: iterate through the points on the boolean hypercube of dimension = log(round_size), skipping
     *                 every other point. On each iteration, create a Univariate<FF, 2> (an 'edge') for each
     *                 multivariate.
     *   - Inner loop: iterate throught the relations, feeding each relation the present collection of edges. Each
     *                 relation adds a contribution
     *
     * Result: for each relation, a univariate of some degree is computed by accumulating the contributions of each
     * group of edges. These are stored in `univariate_accumulators`. Adding these univariates together, with
     * appropriate scaling factors, produces S_l.
     */
    template <size_t relation_idx = 0> void accumulate_relation_univariates()
    {
        std::get<relation_idx>(relations).add_edge_contribution(extended_edges,
                                                                std::get<relation_idx>(univariate_accumulators));

        // Repeat for the next relation.
        if constexpr (relation_idx + 1 < NUM_RELATIONS) {
            accumulate_relation_univariates<relation_idx + 1>();
        }
    }

    // TODO(Cody): make private
    // TODO(Cody): make uniform with accumulate_relation_univariates
    /**
     * @brief Calculate the contribution of each relation to the expected value of the full Honk relation.
     *
     * @details For each relation, use the purported values (supplied by the prover) of the multivariates to calculate
     * a contribution to the purported value of the full Honk relation. These are stored in `evaluations`. Adding these
     * together, with appropriate scaling factors, produces the expected value of the full Honk relation. This value is
     * checked against the final value of the target total sum (called sigma_0 in the thesis).
     */
    template <size_t relation_idx = 0>
    // TODO(Cody): Input should be an array? Then challenge container has to know array length.
    void accumulate_relation_evaluations(std::vector<FF>& purported_evaluations)
    {
        std::get<relation_idx>(relations).add_full_relation_value_contribution(purported_evaluations,
                                                                               evaluations[relation_idx]);

        // Repeat for the next relation.
        if constexpr (relation_idx + 1 < NUM_RELATIONS) {
            accumulate_relation_evaluations<relation_idx + 1>(purported_evaluations);
        }
    }

    /**
     * @brief After executing each widget on each edge, producing a tuple of univariates of differing lenghths,
     * extend all univariates to the max of the lenghths required by the largest relation.
     *
     * @tparam relation_idx
     */
    template <size_t relation_idx = 0> void extend_univariate_accumulators()
    {
        extended_univariates[relation_idx] =
            std::get<relation_idx>(barycentric_utils).extend(std::get<relation_idx>(univariate_accumulators));

        // Repeat for the next relation.
        if constexpr (relation_idx + 1 < NUM_RELATIONS) {
            extend_univariate_accumulators<relation_idx + 1>();
        }
    }

    /**
     * @brief Return the evaluations of the univariate restriction (S_l(X_l) in the thesis) at num_multivariates-many
     * values. Most likely this will end up being S_l(0), ... , S_l(t-1) where t is around 12. At the end, reset all
     * univariate accumulators to be zero.
     */
    Univariate<FF, MAX_RELATION_LENGTH> compute_univariate(auto& polynomials, FF& relation_separator_challenge)
    {
        for (size_t edge_idx = 0; edge_idx < round_size; edge_idx += 2) {
            extend_edges(polynomials, edge_idx);
            accumulate_relation_univariates<>();
        }

        auto result = batch_over_relations<Univariate<FF, MAX_RELATION_LENGTH>>(univariate_accumulators,
                                                                                relation_separator_challenge);

        reset_accumulators<>();

        return result;
    }

    FF compute_full_honk_relation_purported_value(std::vector<FF>& purported_evaluations,
                                                  FF& relation_separator_challenge)
    {
        accumulate_relation_evaluations<>(purported_evaluations);

        // IMPROVEMENT(Cody): Reuse functions from univariate_accumulators batching?
        FF running_challenge = 1;
        FF output = 0;
        for (auto& evals : evaluations) {
            output += evals * running_challenge;
            running_challenge *= relation_separator_challenge;
        }

        return output;
    }

    bool check_sum(Univariate<FF, MAX_RELATION_LENGTH>& univariate)
    {
        FF total_sum = univariate.value_at(0) + univariate.value_at(1);
        bool sumcheck_round_failed = (target_total_sum != total_sum);
        round_failed = round_failed || sumcheck_round_failed;
        return !sumcheck_round_failed;
    };

    FF compute_next_target_sum(Univariate<FF, MAX_RELATION_LENGTH>& univariate, FF& round_challenge)
    {
        // IMPROVEMENT(Cody): Use barycentric static method, maybe implement evaluation as member
        // function on Univariate.
        auto barycentric = BarycentricData<FF, MAX_RELATION_LENGTH, MAX_RELATION_LENGTH>();
        target_total_sum = barycentric.evaluate(univariate, round_challenge);
        return target_total_sum;
    }
};
} // namespace honk::sumcheck
