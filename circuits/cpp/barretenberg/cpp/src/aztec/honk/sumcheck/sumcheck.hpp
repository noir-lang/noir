#include "sumcheck_round.hpp"
#include "polynomials/univariate.hpp"
#include "../flavor/flavor.hpp"
#include <algorithm>
namespace honk::sumcheck {
template <class Multivariates, class ChallengeContainer, template <class> class... Relations> class Sumcheck {
    using FF = typename Multivariates::FF;

  public:
    Multivariates multivariates;
    static constexpr size_t multivariate_d = Multivariates::multivariate_d; // number of variables
    static constexpr size_t multivariate_n = Multivariates::multivariate_n; // 2^d

    std::array<FF, Multivariates::num> purported_evaluations;
    ChallengeContainer challenges;
    SumcheckRound<FF, Multivariates::num, Relations...> round;

    // prover instantiates sumcheck with multivariates
    Sumcheck(Multivariates multivariates, ChallengeContainer challenges)
        : multivariates(multivariates)
        , challenges(challenges)
        , round(Multivariates::num, std::tuple(Relations<FF>(challenges)...)){};

    // verifier instantiates with challenges alone
    explicit Sumcheck(ChallengeContainer challenges)
        : challenges(challenges)
        , round(Multivariates::num, std::tuple(Relations<FF>(challenges)...)){};

    /**
     * @brief Compute univariate restriction place in transcript, generate challenge, fold,... repeat until final round,
     * then compute multivariate evaluations and place in transcript.
     *
     * @details
     */
    void execute_prover()
    {
        std::array<FF, multivariate_d> round_challenges;
        std::fill(round_challenges.begin(), round_challenges.end(), 0);

        // First round
        // This populates multivariates.folded_polynomials.
        FF relation_separator_challenge = challenges.get_relation_separator_challenge();
        challenges.transcript.add(
            round.compute_univariate(multivariates.full_polynomials, relation_separator_challenge));
        // IMPROVEMENT(Cody): Could move building of this list into challenge container?
        round_challenges[0] = challenges.get_sumcheck_round_challenge(0);
        multivariates.fold(multivariates.full_polynomials, multivariate_n, round_challenges[0]);

        // All but final round
        // We operate on multivariates.full_polynomials in place.
        for (size_t round_idx = 1; round_idx < multivariate_d; round_idx++) {
            challenges.transcript.add(
                round.compute_univariate(multivariates.folded_polynomials, relation_separator_challenge));
            round_challenges[round_idx] = challenges.get_sumcheck_round_challenge(round_idx);
            multivariates.fold(multivariates.folded_polynomials, multivariate_n, round_challenges[round_idx]);
        }

        // Final round
        challenges.transcript.add(multivariates.batch_evaluate(round_challenges));
    };

    /**
     * @brief Extract round univariate, check sum, generate challenge, compute next target sum..., repeat until final
     * round, then use purported evaluations to generate purported full Honk relation value and check against final
     * target sum.
     */
    bool execute_verifier()
    {
        bool verified(true);

        // All but final round.
        // target_total_sum is initialized to zero then mutated in place.
        for (size_t round_idx = 0; round_idx < multivariate_d; round_idx++) {
            auto round_univariate = challenges.get_sumcheck_round_univariate(round_idx);
            verified = verified && round.check_sum(round_univariate);
            FF round_challenge = challenges.get_sumcheck_round_challenge(round_idx);
            round.compute_next_target_sum(round_univariate, round_challenge);
        }

        // Final round
        std::vector<FF> purported_evaluations = challenges.get_sumcheck_purported_evaluations();
        FF relation_separator_challenge = challenges.get_relation_separator_challenge();
        FF full_honk_relation_purported_value =
            round.compute_full_honk_relation_purported_value(purported_evaluations, relation_separator_challenge);
        verified = verified && (full_honk_relation_purported_value == round.target_total_sum);
        return verified;
    };
};
} // namespace honk::sumcheck
