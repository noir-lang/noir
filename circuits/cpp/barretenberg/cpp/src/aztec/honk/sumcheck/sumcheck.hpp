#pragma once
#include "common/serialize.hpp"
#include "proof_system/types/polynomial_manifest.hpp"
#include <honk/utils/public_inputs.hpp>
#include "common/throw_or_abort.hpp"
#include "ecc/curves/bn254/fr.hpp"
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
    Sumcheck(Multivariates& multivariates, Transcript& transcript)
        : multivariates(multivariates)
        , transcript(transcript)
        , round(multivariates.multivariate_n, std::tuple(Relations<FF>()...)){};

    // verifier instantiates with challenges alone
    explicit Sumcheck(Transcript& transcript)
        : multivariates(transcript)
        , transcript(transcript)
        , round(std::tuple(Relations<FF>()...)){};

    /**
     * @brief Get all the challenges and computed parameters used in sumcheck in a convenient format
     *
     * @return RelationParameters<FF>
     */
    RelationParameters<FF> retrieve_proof_parameters()
    {
        const FF alpha = FF::serialize_from_buffer(transcript.get_challenge("alpha").begin());
        const FF beta = FF::serialize_from_buffer(transcript.get_challenge("beta").begin());
        const FF gamma = FF::serialize_from_buffer(transcript.get_challenge("beta", 1).begin());
        const auto public_input_size_vector = transcript.get_element("public_input_size");
        const size_t public_input_size = (static_cast<size_t>(public_input_size_vector[0]) << 24) |
                                         (static_cast<size_t>(public_input_size_vector[1]) << 16) |
                                         (static_cast<size_t>(public_input_size_vector[2]) << 8) |

                                         static_cast<size_t>(public_input_size_vector[3]);
        const auto circut_size_vector = transcript.get_element("circuit_size");
        const size_t n = (static_cast<size_t>(circut_size_vector[0]) << 24) |
                         (static_cast<size_t>(circut_size_vector[1]) << 16) |
                         (static_cast<size_t>(circut_size_vector[2]) << 8) | static_cast<size_t>(circut_size_vector[3]);
        std::vector<FF> public_inputs = many_from_buffer<FF>(transcript.get_element("public_inputs"));
        ASSERT(public_inputs.size() == public_input_size);
        FF public_input_delta = honk::compute_public_input_delta<FF>(public_inputs, beta, gamma, n);
        const RelationParameters<FF> relation_parameters = RelationParameters<FF>{
            .alpha = alpha, .beta = beta, .gamma = gamma, .public_input_delta = public_input_delta
        };
        return relation_parameters;
    }
    /**
     * @brief Compute univariate restriction place in transcript, generate challenge, fold,... repeat until final round,
     * then compute multivariate evaluations and place in transcript.
     *
     * @details
     */
    void execute_prover()
    {
        // First round
        // This populates multivariates.folded_polynomials.

        const auto relation_parameters = retrieve_proof_parameters();
        auto round_univariate = round.compute_univariate(multivariates.full_polynomials, relation_parameters);
        transcript.add_element("univariate_" + std::to_string(multivariates.multivariate_d),
                               round_univariate.to_buffer());
        std::string challenge_label = "u_" + std::to_string(multivariates.multivariate_d);
        transcript.apply_fiat_shamir(challenge_label);
        FF round_challenge = FF::serialize_from_buffer(transcript.get_challenge(challenge_label).begin());
        multivariates.fold(multivariates.full_polynomials, multivariates.multivariate_n, round_challenge);
        round.round_size = round.round_size >> 1; // TODO(Cody): Maybe fold should do this and release memory?

        // All but final round
        // We operate on multivariates.folded_polynomials in place.
        for (size_t round_idx = 1; round_idx < multivariates.multivariate_d; round_idx++) {
            // Write the round univariate to the transcript
            round_univariate = round.compute_univariate(multivariates.folded_polynomials, relation_parameters);
            transcript.add_element("univariate_" + std::to_string(multivariates.multivariate_d - round_idx),
                                   round_univariate.to_buffer());
            challenge_label = "u_" + std::to_string(multivariates.multivariate_d - round_idx);
            transcript.apply_fiat_shamir(challenge_label);
            FF round_challenge = FF::serialize_from_buffer(transcript.get_challenge(challenge_label).begin());
            multivariates.fold(multivariates.folded_polynomials, round.round_size, round_challenge);
            round.round_size = round.round_size >> 1;
        }

        // Final round
        transcript.add_element("multivariate_evaluations",
                               // TODO(Cody): will need to do this programatically.
                               to_buffer(std::array<FF, waffle::STANDARD_HONK_TOTAL_NUM_POLYS>(
                                   { multivariates.folded_polynomials[0][0],
                                     multivariates.folded_polynomials[1][0],
                                     multivariates.folded_polynomials[2][0],
                                     multivariates.folded_polynomials[3][0],
                                     multivariates.folded_polynomials[4][0],
                                     multivariates.folded_polynomials[5][0],
                                     multivariates.folded_polynomials[6][0],
                                     multivariates.folded_polynomials[7][0],
                                     multivariates.folded_polynomials[8][0],
                                     multivariates.folded_polynomials[9][0],
                                     multivariates.folded_polynomials[10][0],
                                     multivariates.folded_polynomials[11][0],
                                     multivariates.folded_polynomials[12][0],
                                     multivariates.folded_polynomials[13][0],
                                     multivariates.folded_polynomials[14][0],
                                     multivariates.folded_polynomials[15][0],
                                     multivariates.folded_polynomials[16][0],
                                     multivariates.folded_polynomials[17][0],
                                     multivariates.folded_polynomials[18][0] })));
    };

    /**
     * @brief Extract round univariate, check sum, generate challenge, compute next target sum..., repeat until final
     * round, then use purported evaluations to generate purported full Honk relation value and check against final
     * target sum.
     */
    bool execute_verifier()
    {
        bool verified(true);

        const auto relation_parameters = retrieve_proof_parameters();
        // All but final round.
        // target_total_sum is initialized to zero then mutated in place.

        if (multivariates.multivariate_d == 0) {
            throw_or_abort("Number of variables in multivariate is 0.");
        }

        for (size_t round_idx = 0; round_idx < multivariates.multivariate_d; round_idx++) {
            // Obtain the round univariate from the transcript
            auto round_univariate = Univariate<FF, MAX_RELATION_LENGTH>::serialize_from_buffer(
                &transcript.get_element("univariate_" + std::to_string(multivariates.multivariate_d - round_idx))[0]);
            bool checked = round.check_sum(round_univariate);
            verified = verified && checked;
            FF round_challenge = FF::serialize_from_buffer(
                transcript.get_challenge("u_" + std::to_string(multivariates.multivariate_d - round_idx)).begin());

            round.compute_next_target_sum(round_univariate, round_challenge);

            if (!verified) {
                return false;
            }
        }

        // Final round
        auto purported_evaluations = transcript.get_field_element_vector("multivariate_evaluations");
        FF full_honk_relation_purported_value =
            round.compute_full_honk_relation_purported_value(purported_evaluations, relation_parameters);
        verified = verified && (full_honk_relation_purported_value == round.target_total_sum);
        return verified;
    };
};
} // namespace honk::sumcheck
