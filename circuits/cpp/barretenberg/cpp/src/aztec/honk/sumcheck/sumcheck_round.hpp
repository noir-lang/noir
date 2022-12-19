#include <common/log.hpp>
#include <cstddef>
#include <array>
#include <vector>
#include <tuple>
#include "./sumcheck_types/univariate.hpp"
#include "./sumcheck_types/constraint_manager.hpp"
#include "./sumcheck_types/barycentric_data.hpp"
namespace honk {
namespace sumcheck {

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
    |          |   |          |  where the constraints are G0(Y1, Y2) = Y1 * Y2
    1 -------- 5   2 -------- 6                        and G1(Y1, Y2) = Y1 + Y2.

 G itself is represented by a SumcheckRound class.
 G1, G2 together comprise the ConstraintPack.
 The polynomials Y1, Y2 are stored as two edge groups:
               3     4                         7     8
one containing | and |  and another containing | and | .
               1     2                         5     6
The rationale here is that both folding and evaluation will proceed one edge group at a time.
 */

template <class SumcheckTypes, template <class> class... ConstraintPack> class SumcheckRound {
    using Fr = typename SumcheckTypes::Fr;
    using Multivariates = typename SumcheckTypes::Multivariates;
    using ChallengeContainer = typename SumcheckTypes::ChallengeContainer;

  public:
    size_t round_size;
    bool failed = false;
    Fr target_total_sum;
    Multivariates multivariates;
    std::array<Fr, Multivariates::num> purported_evaluations;
    ConstraintManager<ConstraintPack<Fr>...> constraint_manager; // TODO(cody); move more evals, maybe more, into here
    ChallengeContainer challenges;
    static constexpr size_t SUMCHECK_CONSTRAINT_DEGREE_PLUS_ONE =
        5; // TODO(luke): This value is independently defined in multiple locations
    BarycentricData<Fr, 2, SUMCHECK_CONSTRAINT_DEGREE_PLUS_ONE> barycentric =
        BarycentricData<Fr, 2, SUMCHECK_CONSTRAINT_DEGREE_PLUS_ONE>();
    std::array<Univariate<Fr, SUMCHECK_CONSTRAINT_DEGREE_PLUS_ONE>, Multivariates::num> edge_extensions;
    std::array<Univariate<Fr, SUMCHECK_CONSTRAINT_DEGREE_PLUS_ONE>, Multivariates::num> extended_univariates;
    static constexpr size_t NUM_CONSTRAINTS = sizeof...(ConstraintPack);

    SumcheckRound(Multivariates multivariates, ChallengeContainer challenges)
        : multivariates(multivariates)
        , challenges(challenges)
    {
        // TODO: test edge case
        // Here (and everywhere?) multivariate_n is assumed to be a power of 2
        // will be halved after each round
        round_size = multivariates.multivariate_n;
    }

    SumcheckRound(std::array<Fr, Multivariates::num> purported_evaluations, ChallengeContainer challenges)
        : purported_evaluations(purported_evaluations)
        , challenges(challenges)
    {}

    /**
     * @brief Given a tuple t = (t_0, t_1, ..., t_{NUM_CONSTRAINTS-1}) and a challenge α,
     * modify the tuple in place to (t_0, αt_1, ..., α^{NUM_CONSTRAINTS-1}t_{NUM_CONSTRAINTS-1}).
     */
    template <size_t idx = 0> void scale_tuple(auto& tuple, Fr challenge, Fr running_challenge)
    {
        std::get<idx>(tuple) *= running_challenge;
        running_challenge *= challenge;
        if constexpr (idx + 1 < NUM_CONSTRAINTS) {
            scale_tuple<idx + 1>(tuple, challenge, running_challenge);
        }
    };
    /**
     * @brief Given a tuple t = (t_0, t_1, ..., t_{NUM_CONSTRAINTS-1}) and a challenge α,
     * return t_0 + αt_1 + ... + α^{NUM_CONSTRAINTS-1}t_{NUM_CONSTRAINTS-1}).
     */
    template <typename T> T batch_over_constraints(auto& tuple, Fr challenge)
    {
        Fr running_challenge = 1;
        scale_tuple<>(tuple, challenge, running_challenge);
        extend_univariate_accumulators<>();
        auto result = T();
        // T result = std::apply([&](auto... v) { return (v + ...); }, tuple);
        for (size_t i = 0; i < NUM_CONSTRAINTS; ++i) {
            result += extended_univariates[i];
        }
        return result;
    }

    /**
     * @brief Evaluate some constraints by evaluating each edge in the edge group at
     * Univariate::length-many values. Store each value separately in the corresponding
     * entry of constraint_evals.
     *
     * @details Should only be called externally with constraint_idx equal to 0.
     *
     */
    void extend_edges(std::array<Fr*, Multivariates::num> polynomial, size_t edge_idx)
    {
        for (size_t idx = 0; idx < Multivariates::num; idx++) {
            auto edge = Univariate<Fr, 2>({ polynomial[idx][edge_idx], polynomial[idx][edge_idx + 1] });
            edge_extensions[idx] = barycentric.extend(edge);
        }
    }

    // TODO(cody): make private
    template <size_t constraint_idx = 0> void accumulate_constraint_univariates()
    {
        std::get<constraint_idx>(constraint_manager.constraints)
            .add_edge_contribution(edge_extensions,
                                   std::get<constraint_idx>(constraint_manager.univariate_accumulators));

        // Repeat for the next constraint.
        if constexpr (constraint_idx + 1 < NUM_CONSTRAINTS) {
            accumulate_constraint_univariates<constraint_idx + 1>();
        }
    }

    // TODO(cody): make private
    // TODO(cody): make uniform with univariates
    template <size_t constraint_idx = 0>
    void accumulate_constraint_evaluations(std::array<Fr, NUM_CONSTRAINTS>& constraint_evaluations)
    {
        std::get<constraint_idx>(constraint_manager.constraints)
            .add_full_constraint_value_contribution(purported_evaluations, constraint_evaluations[constraint_idx]);

        // Repeat for the next constraint.
        if constexpr (constraint_idx + 1 < NUM_CONSTRAINTS) {
            accumulate_constraint_evaluations<constraint_idx + 1>(constraint_evaluations);
        }
    }

    /**
     * @brief After executing each widget on each edge, producing a tuple of univariates of differing lenghts,
     * extend all univariates to the max of the lenghts required by the largest constraint.
     *
     * @tparam constraint_idx
     */
    template <size_t constraint_idx = 0> void extend_univariate_accumulators()
    {
        extended_univariates[constraint_idx] =
            std::get<constraint_idx>(constraint_manager.constraints)
                .barycentric.extend(std::get<constraint_idx>(constraint_manager.univariate_accumulators));

        // Repeat for the next constraint.
        if constexpr (constraint_idx + 1 < NUM_CONSTRAINTS) {
            extend_univariate_accumulators<constraint_idx + 1>();
        }
    }

    /**
     * @brief In the first round, return the evaluations of the univariate restriction S(X_l) at EdgeGroup::length-many
     * values. Most likely this will end up being S(0), ... , S(t-1) where t is around 12.
     *
     * @details We have a separate function for the first round so we can halve the memory
     * usage of the sumcheck round by only copy the results after folding.
     *
     * @param edges
     * @param challenges
     * @return Univariate<Fr, SUMCHECK_CONSTRAINT_DEGREE_PLUS_ONE>
     *
     */
    Univariate<Fr, SUMCHECK_CONSTRAINT_DEGREE_PLUS_ONE> compute_initial_univariate_restriction(
        Multivariates& multivariates, ChallengeContainer& challenges)
    {
        for (size_t edge_idx = 0; edge_idx < round_size; edge_idx += 2) {
            extend_edges(multivariates.full_polynomials, edge_idx);
            accumulate_constraint_univariates<>();
        }

        Fr running_challenge(1);
        Fr challenge = challenges.get_constraint_separator_challenge();
        auto result = batch_over_constraints<Univariate<Fr, SUMCHECK_CONSTRAINT_DEGREE_PLUS_ONE>>(
            constraint_manager.univariate_accumulators, challenge);

        // at this point, we are able to
        // - add these coefficients to the transcript and extract u_l
        // - partially evaluate all multivariates in u_l
        return result;
    }

    /**
     * @brief Return the evaluations of the univariate restriction S_l(X_l) at EdgeGroup::length-many values.
     * Most likely this will end up being S_l(0), ... , S_l(t-1) where t is around 12.
     *
     * @param edges
     * @param challenges
     * @return Univariate<Fr, SUMCHECK_CONSTRAINT_DEGREE_PLUS_ONE>
     *
     */
    Univariate<Fr, SUMCHECK_CONSTRAINT_DEGREE_PLUS_ONE> compute_univariate_restriction(Multivariates& multivariates,
                                                                                       ChallengeContainer& challenges)
    {
        // For each edge index, iterate over all constraints, accumulating for each constraint the contribution
        // to each of desired evaluations of S_l.
        std::tuple<Univariate<Fr, ConstraintPack<Fr>::CONSTRAINT_LENGTH>...> constraint_univariates =
            std::tuple(ConstraintPack<Fr>::CONSTRAINT_LENGTH...);

        for (size_t edge_idx = 0; edge_idx < round_size; edge_idx += 2) {
            extend_edges(multivariates.folded_multivariates, edge_idx);
            accumulate_constraint_univariates<>(constraint_univariates);
        }

        // Construct the univariate restriction
        Fr running_challenge(1);
        Fr challenge = challenges.get_constraint_separator_challenge();
        Univariate<Fr, SUMCHECK_CONSTRAINT_DEGREE_PLUS_ONE> result({ 0 }); // need to initialize to 0
        for (auto& univariate : constraint_univariates) {
            result += univariate * running_challenge;
            running_challenge *= challenge;
        }

        // - add these coefficients to the transcript and extract u_l
        // - partially evaluate all multivariates in u_l
        return result;
    }

    Fr compute_full_honk_constraint_purported_value(ChallengeContainer& challenges)
    {
        // TODO(cody): Reuse functions from univariate_accumulators batching?
        std::array<Fr, NUM_CONSTRAINTS> constraint_evaluations{ { 0 } };
        accumulate_constraint_evaluations<>(constraint_evaluations);

        Fr running_challenge(1);
        Fr challenge = challenges.get_constraint_separator_challenge();
        Fr output(0);
        for (auto& evals : constraint_evaluations) {
            output += evals * running_challenge;
            running_challenge *= challenge;
        }

        return output;
    }

    bool check_sum(Univariate<Fr, SUMCHECK_CONSTRAINT_DEGREE_PLUS_ONE>& univariate_restriction)
    {
        // S_l(0) + S_l(1)
        Fr total_sum = univariate_restriction.at(0) + univariate_restriction.at(1);
        bool sumcheck_round_passes = (target_total_sum == total_sum); // an assert_equal
        return sumcheck_round_passes;
    };

    /**
     * @brief Have ChallengeContainer sample the next univariate evalution challenge u_l and the resulting univariate
     * value sigma_l of the function S_l.
     * @param evals
     * @param challenges
     */
    Fr compute_challenge_and_evaluation(Univariate<Fr, SUMCHECK_CONSTRAINT_DEGREE_PLUS_ONE>& univariate_restriction,
                                        ChallengeContainer& challenges)
    {
        // add challenges to transcript and run Fiat-Shamir
        Fr challenge = challenges.get_sumcheck_round_challenge(univariate_restriction);
        auto barycentric = BarycentricData<Fr,
                                           Univariate<Fr, SUMCHECK_CONSTRAINT_DEGREE_PLUS_ONE>::num_evals,
                                           Univariate<Fr, SUMCHECK_CONSTRAINT_DEGREE_PLUS_ONE>::num_evals>();
        target_total_sum = barycentric.evaluate(univariate_restriction, challenge);
        return challenge;
    }

    bool execute_first_round()
    {
        Univariate<Fr, SUMCHECK_CONSTRAINT_DEGREE_PLUS_ONE> univariate_restriction =
            compute_initial_univariate_restriction(multivariates, challenges);
        // evaluate univariate restriction and challenge and check for equality with target value
        failed = !check_sum(univariate_restriction);

        if (failed) {
            // TODO: use to set composer.failed?
            return false;
        }
        // compute univariate evaluation challenge and update the target value
        // for the next call to check_sum
        Fr round_challenge = compute_challenge_and_evaluation(univariate_restriction, challenges);
        multivariates.fold(round_size, round_challenge);
        round_size /= 2;

        return true;
    };

    bool execute()
    {
        Univariate<Fr, SUMCHECK_CONSTRAINT_DEGREE_PLUS_ONE> univariate_restriction =
            compute_univariate_restriction(multivariates, challenges);
        // evaluate univariate restriction and challenge and check for equality with target value
        failed = !check_sum(univariate_restriction);

        if (failed) {
            // TODO: use to set composer.failed?
            return false;
        }
        // compute univariate evaluation challenge and update the target value
        // for the next call to check_sum
        Fr round_challenge = compute_challenge_and_evaluation(univariate_restriction, challenges);
        multivariates.fold(round_size, round_challenge);
        round_size /= 2;

        return true;
    };
};
} // namespace sumcheck
} // namespace honk
