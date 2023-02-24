#pragma once
#include "common/serialize.hpp"
#include <array>
#include <honk/utils/public_inputs.hpp>
#include "common/throw_or_abort.hpp"
#include "sumcheck_round.hpp"
#include "polynomials/univariate.hpp"
#include <proof_system/flavor/flavor.hpp>
#include <algorithm>
#include <cstddef>
#include <span>
#include <string>
#include <vector>
#include <honk/proof_system/prover.hpp>
namespace honk::sumcheck {
template <typename FF, class Transcript, template <class> class... Relations> class Sumcheck {

  public:
    // TODO(luke): this value is needed here but also lives in sumcheck_round
    static constexpr size_t MAX_RELATION_LENGTH = std::max({ Relations<FF>::RELATION_LENGTH... });

    std::array<FF, bonk::StandardArithmetization::NUM_POLYNOMIALS> purported_evaluations;
    Transcript& transcript;
    const size_t multivariate_n;
    const size_t multivariate_d;
    SumcheckRound<FF, bonk::StandardArithmetization::NUM_POLYNOMIALS, Relations...> round;

    /**
    *
    * @brief (folded_polynomials) Suppose the Honk polynomials (multilinear in d variables) are called P_1, ..., P_N.
    * At initialization,
    * we think of these as lying in a two-dimensional array, where each column records the value of one P_i on H^d.
    * After the first round, the array will be updated ('folded'), so that the first n/2 rows will represent the
    * evaluations P_i(u0, X1, ..., X_{d-1}) as a low-degree extension on H^{d-1}. In reality, we elude copying all
    * of the polynomial-defining data by only populating folded_multivariates after the first round. I.e.:

        We imagine all of the defining polynomial data in a matrix like this:
                    | P_1 | P_2 | P_3 | P_4 | ... | P_N | N = number of multivariatesk
                    |-----------------------------------|
          group 0 --|  *  |  *  |  *  |  *  | ... |  *  | vertex 0
                  \-|  *  |  *  |  *  |  *  | ... |  *  | vertex 1
          group 1 --|  *  |  *  |  *  |  *  | ... |  *  | vertex 2
                  \-|  *  |  *  |  *  |  *  | ... |  *  | vertex 3
                    |  *  |  *  |  *  |  *  | ... |  *  |
        group m-1 --|  *  |  *  |  *  |  *  | ... |  *  | vertex n-2
                  \-|  *  |  *  |  *  |  *  | ... |  *  | vertex n-1
            m = n/2
                                        *
            Each group consists of N edges |, and our construction of univariates and folding
                                        *
            operations naturally operate on these groups of edges

    *
    * NOTE: With ~40 columns, prob only want to allocate 256 EdgeGroup's at once to keep stack under 1MB?
    * TODO(Cody): might want to just do C-style multidimensional array? for guaranteed adjacency?
    */
    std::array<std::vector<FF>, bonk::StandardArithmetization::NUM_POLYNOMIALS> folded_polynomials;

    // prover instantiates sumcheck with circuit size and transcript
    Sumcheck(size_t multivariate_n, Transcript& transcript)
        : transcript(transcript)
        , multivariate_n(multivariate_n)
        , multivariate_d(numeric::get_msb(multivariate_n))
        , round(multivariate_n, std::tuple(Relations<FF>()...))
    {
        for (auto& polynomial : folded_polynomials) {
            polynomial.resize(multivariate_n >> 1);
        }
    };

    // verifier instantiates with transcript alone
    explicit Sumcheck(Transcript& transcript)
        : transcript(transcript)
        , multivariate_n([](std::vector<uint8_t> buffer) {
            return static_cast<size_t>(buffer[3]) + (static_cast<size_t>(buffer[2]) << 8) +
                   (static_cast<size_t>(buffer[1]) << 16) + (static_cast<size_t>(buffer[0]) << 24);
        }(transcript.get_element("circuit_size")))
        , multivariate_d(numeric::get_msb(multivariate_n))
        , round(std::tuple(Relations<FF>()...))
    {
        for (auto& polynomial : folded_polynomials) {
            polynomial.resize(multivariate_n >> 1);
        }
    };

    /**
     * @brief Get all the challenges and computed parameters used in sumcheck in a convenient format
     *
     * @return RelationParameters<FF>
     */
    RelationParameters<FF> retrieve_proof_parameters()
    {
        const FF alpha = FF::serialize_from_buffer(transcript.get_challenge("alpha").begin());
        const FF zeta = FF::serialize_from_buffer(transcript.get_challenge("alpha", 1).begin());
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
            .zeta = zeta, .alpha = alpha, .beta = beta, .gamma = gamma, .public_input_delta = public_input_delta
        };
        return relation_parameters;
    }

    /**
     * @brief Compute univariate restriction place in transcript, generate challenge, fold,... repeat until final round,
     * then compute multivariate evaluations and place in transcript.
     *
     * @details
     */
    void execute_prover(auto full_polynomials) // pass by value, not by reference
    {
        // First round
        // This populates folded_polynomials.

        const auto relation_parameters = retrieve_proof_parameters();
        PowUnivariate<FF> pow_univariate(relation_parameters.zeta);

        auto round_univariate = round.compute_univariate(full_polynomials, relation_parameters, pow_univariate);
        transcript.add_element("univariate_0", round_univariate.to_buffer());
        std::string challenge_label = "u_0";
        transcript.apply_fiat_shamir(challenge_label);
        FF round_challenge = FF::serialize_from_buffer(transcript.get_challenge(challenge_label).begin());
        fold(full_polynomials, multivariate_n, round_challenge);
        pow_univariate.partially_evaluate(round_challenge);
        round.round_size = round.round_size >> 1; // TODO(Cody): Maybe fold should do this and release memory?

        // All but final round
        // We operate on folded_polynomials in place.
        for (size_t round_idx = 1; round_idx < multivariate_d; round_idx++) {
            // Write the round univariate to the transcript
            round_univariate = round.compute_univariate(folded_polynomials, relation_parameters, pow_univariate);
            transcript.add_element("univariate_" + std::to_string(round_idx), round_univariate.to_buffer());
            challenge_label = "u_" + std::to_string(round_idx);
            transcript.apply_fiat_shamir(challenge_label);
            FF round_challenge = FF::serialize_from_buffer(transcript.get_challenge(challenge_label).begin());
            fold(folded_polynomials, round.round_size, round_challenge);
            pow_univariate.partially_evaluate(round_challenge);
            round.round_size = round.round_size >> 1;
        }

        // Final round: Extract multivariate evaluations from folded_polynomials and add to transcript
        std::array<FF, bonk::StandardArithmetization::NUM_POLYNOMIALS> multivariate_evaluations;
        for (size_t i = 0; i < bonk::StandardArithmetization::NUM_POLYNOMIALS; ++i) {
            multivariate_evaluations[i] = folded_polynomials[i][0];
        }
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

        const auto relation_parameters = retrieve_proof_parameters();
        PowUnivariate<FF> pow_univariate(relation_parameters.zeta);
        // All but final round.
        // target_total_sum is initialized to zero then mutated in place.

        if (multivariate_d == 0) {
            throw_or_abort("Number of variables in multivariate is 0.");
        }

        for (size_t round_idx = 0; round_idx < multivariate_d; round_idx++) {
            // Obtain the round univariate from the transcript
            auto round_univariate = Univariate<FF, MAX_RELATION_LENGTH>::serialize_from_buffer(
                &transcript.get_element("univariate_" + std::to_string(round_idx))[0]);
            bool checked = round.check_sum(round_univariate, pow_univariate);
            verified = verified && checked;
            FF round_challenge =
                FF::serialize_from_buffer(transcript.get_challenge("u_" + std::to_string(round_idx)).begin());

            round.compute_next_target_sum(round_univariate, round_challenge, pow_univariate);
            pow_univariate.partially_evaluate(round_challenge);

            if (!verified) {
                return false;
            }
        }

        // Final round
        auto purported_evaluations = transcript.get_field_element_vector("multivariate_evaluations");
        FF full_honk_relation_purported_value = round.compute_full_honk_relation_purported_value(
            purported_evaluations, relation_parameters, pow_univariate);
        verified = verified && (full_honk_relation_purported_value == round.target_total_sum);
        return verified;
    };

    // TODO(Cody): Rename. fold is not descriptive, and it's already in use in the Gemini context.
    //             Probably just call it partial_evaluation?
    /**
     * @brief Evaluate at the round challenge and prepare class for next round.
     * Illustration of layout in example of first round when d==3 (showing just one Honk polynomial,
     * i.e., what happens in just one column of our two-dimensional array):
     *
     * groups    vertex terms              collected vertex terms               groups after folding
     *     g0 -- v0 (1-X0)(1-X1)(1-X2) --- (v0(1-X0) + v1 X0) (1-X1)(1-X2) ---- (v0(1-u0) + v1 u0) (1-X1)(1-X2)
     *        \- v1   X0  (1-X1)(1-X2) --/                                  --- (v2(1-u0) + v3 u0)   X1  (1-X2)
     *     g1 -- v2 (1-X0)  X1  (1-X2) --- (v2(1-X0) + v3 X0)   X1  (1-X2)-/ -- (v4(1-u0) + v5 u0) (1-X1)  X2
     *        \- v3   X0    X1  (1-X2) --/                                  / - (v6(1-u0) + v7 u0)   X1    X2
     *     g2 -- v4 (1-X0)(1-X1)  X2   --- (v4(1-X0) + v5 X0) (1-X1)  X2  -/ /
     *        \- v5   X0  (1-X1)  X2   --/                                  /
     *     g3 -- v6 (1-X0)  X1    X2   --- (v6(1-X0) + v7 X0)   X1    X2  -/
     *        \- v7   X0    X1    X2   --/
     *
     * @param challenge
     */
    void fold(auto& polynomials, size_t round_size, FF round_challenge)
    {
        // after the first round, operate in place on folded_polynomials
        for (size_t j = 0; j < polynomials.size(); ++j) {
            // for (size_t j = 0; j < bonk::StandardArithmetization::NUM_POLYNOMIALS; ++j) {
            for (size_t i = 0; i < round_size; i += 2) {
                folded_polynomials[j][i >> 1] =
                    polynomials[j][i] + round_challenge * (polynomials[j][i + 1] - polynomials[j][i]);
            }
        }
    };
};
} // namespace honk::sumcheck
