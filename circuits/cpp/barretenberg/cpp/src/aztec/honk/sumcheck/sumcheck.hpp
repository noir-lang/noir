#include "common/serialize.hpp"
#include "sumcheck_round.hpp"
#include "polynomials/univariate.hpp"
#include <proof_system/flavor/flavor.hpp>
#include <algorithm>
#include <cstddef>
#include <string>
#include <vector>
namespace honk::sumcheck {
template <class Multivariates, class Transcript, template <class> class... Relations> class Sumcheck {
    using FF = typename Multivariates::FF;

  public:
    Multivariates multivariates;
    // TODO(luke): this value is needed here but also lives in sumcheck_round
    static constexpr size_t MAX_RELATION_LENGTH = std::max({ Relations<FF>::RELATION_LENGTH... });

    std::array<FF, Multivariates::num> purported_evaluations;
    Transcript& transcript;
    SumcheckRound<FF, Multivariates::num, Relations...> round;

    // prover instantiates sumcheck with multivariates
    Sumcheck(Multivariates multivariates, Transcript& transcript)
        : multivariates(multivariates)
        , transcript(transcript)
        , round(Multivariates::num, std::tuple(Relations<FF>()...)){};

    // verifier instantiates with challenges alone
    explicit Sumcheck(Transcript& transcript)
        : multivariates(transcript)
        , transcript(transcript)
        , round(Multivariates::num, std::tuple(Relations<FF>()...)){};

    /**
     * @brief Compute univariate restriction place in transcript, generate challenge, fold,... repeat until final round,
     * then compute multivariate evaluations and place in transcript.
     *
     * @details
     */
    void execute_prover()
    {
        std::vector<FF> round_challenges;
        round_challenges.reserve(multivariates.multivariate_d);
        std::fill(round_challenges.begin(), round_challenges.end(), 0);

        // First round
        // This populates multivariates.folded_polynomials.
        FF relation_separator_challenge = transcript.get_mock_challenge();
        auto round_univariate = round.compute_univariate(multivariates.full_polynomials, relation_separator_challenge);
        transcript.add_element("univariate_" + std::to_string(multivariates.multivariate_d),
                               round_univariate.to_buffer());
        transcript.apply_fiat_shamir("u_" + std::to_string(multivariates.multivariate_d));
        multivariates.fold(multivariates.full_polynomials, multivariates.multivariate_n, round_challenges[0]);

        // All but final round
        // We operate on multivariates.folded_polynomials in place.
        for (size_t round_idx = 1; round_idx < multivariates.multivariate_d; round_idx++) {
            // Write the round univariate to the transcript
            round_univariate = round.compute_univariate(multivariates.folded_polynomials, relation_separator_challenge);
            transcript.add_element("univariate_" + std::to_string(multivariates.multivariate_d - round_idx),
                                   round_univariate.to_buffer());

            transcript.apply_fiat_shamir("u_" + std::to_string(multivariates.multivariate_d - round_idx));
            multivariates.fold(
                multivariates.folded_polynomials, multivariates.multivariate_n, round_challenges[round_idx]);
        }

        // Final round
        // TODO(Cody): Figure out how to actually handle this. Currently done in prover, should happen here prob.
        // auto multivariate_evaluations = multivariates.batch_evaluate();
        // transcript.add_element("multivariate_evaluations", to_buffer(multivariate_evaluations));
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
        for (size_t round_idx = 0; round_idx < multivariates.multivariate_d; round_idx++) {
            // Obtain the round univariate from the transcript
            auto round_univariate = Univariate<FF, MAX_RELATION_LENGTH>::serialize_from_buffer(
                &transcript.get_element("univariate_" + std::to_string(round_idx))[0]);

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
