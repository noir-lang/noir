#include "common/serialize.hpp"
#include "sumcheck_round.hpp"
#include "polynomials/univariate.hpp"
#include "../flavor/flavor.hpp"
#include <algorithm>
#include <cstddef>
#include <string>
#include <vector>
namespace honk::sumcheck {
template <class Multivariates, class Transcript, template <class> class... Relations> class Sumcheck {
    using FF = typename Multivariates::FF;

  public:
    Multivariates multivariates;
    static constexpr size_t multivariate_d = Multivariates::multivariate_d; // number of variables
    static constexpr size_t multivariate_n = Multivariates::multivariate_n; // 2^d
    // TODO(luke): this value is needed here but also lives in sumcheck_round
    static constexpr size_t MAX_RELATION_LENGTH = std::max({ Relations<FF>::RELATION_LENGTH... });

    std::array<FF, Multivariates::num> purported_evaluations;
    Transcript transcript;
    SumcheckRound<FF, Multivariates::num, Relations...> round;

    // prover instantiates sumcheck with multivariates
    Sumcheck(Multivariates multivariates, Transcript transcript)
        : multivariates(multivariates)
        , transcript(transcript)
        , round(Multivariates::num, std::tuple(Relations<FF>()...)){};

    // verifier instantiates with challenges alone
    explicit Sumcheck(Transcript transcript)
        : transcript(transcript)
        , round(Multivariates::num, std::tuple(Relations<FF>()...)){};

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
        FF relation_separator_challenge = transcript.get_mock_challenge();
        auto round_univariate = round.compute_univariate(multivariates.full_polynomials, relation_separator_challenge);
        transcript.add_element("univariate_0", round_univariate.to_buffer());
        // IMPROVEMENT(Cody): Could move building of this list into challenge container?
        round_challenges[0] = transcript.get_mock_challenge();
        multivariates.fold(multivariates.full_polynomials, multivariate_n, round_challenges[0]);

        // All but final round
        // We operate on multivariates.folded_polynomials in place.
        for (size_t round_idx = 1; round_idx < multivariate_d; round_idx++) {
            // Write the round univariate to the transcript
            round_univariate = round.compute_univariate(multivariates.folded_polynomials, relation_separator_challenge);
            transcript.add_element("univariate_" + std::to_string(round_idx), round_univariate.to_buffer());

            round_challenges[round_idx] = transcript.get_mock_challenge();
            multivariates.fold(multivariates.folded_polynomials, multivariate_n, round_challenges[round_idx]);
        }

        // Final round
        // Note: get evaluations from folded_polynomials; don't need batch_evaluate
        auto multivariate_evaluations = multivariates.batch_evaluate(round_challenges);
        transcript.add_element("multivariate_evaluations", to_buffer(multivariate_evaluations));
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
            // Obtain the round univariate from the transcript
            auto round_univariate = Univariate<FF, MAX_RELATION_LENGTH>::serialize_from_buffer(
                transcript.get_element("univariate_" + std::to_string(round_idx)));

            verified = verified && round.check_sum(round_univariate);
            FF round_challenge = transcript.get_mock_challenge();
            round.compute_next_target_sum(round_univariate, round_challenge);
        }

        // Final round
        auto purported_evaluations = transcript.get_field_element_vector("multivariate_evaluations");
        FF relation_separator_challenge = transcript.get_mock_challenge();
        FF full_honk_relation_purported_value =
            round.compute_full_honk_relation_purported_value(purported_evaluations, relation_separator_challenge);
        verified = verified && (full_honk_relation_purported_value == round.target_total_sum);
        return verified;
    };
};
} // namespace honk::sumcheck
