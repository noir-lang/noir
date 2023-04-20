#pragma once
#include "barretenberg/common/serialize.hpp"
#include <array>
#include "barretenberg/honk/sumcheck/relations/relation.hpp"
#include "barretenberg/honk/transcript/transcript.hpp"
#include "barretenberg/honk/utils/grand_product_delta.hpp"
#include "barretenberg/common/throw_or_abort.hpp"
#include "sumcheck_round.hpp"
#include "polynomials/univariate.hpp"
#include "barretenberg/honk/flavor/flavor.hpp"
#include <algorithm>
#include <cstddef>
#include <span>
#include <string>
#include <vector>
#include "barretenberg/honk/proof_system/prover.hpp"
#include "barretenberg/honk/sumcheck/sumcheck_output.hpp"
#include <optional>

namespace proof_system::honk::sumcheck {

template <typename FF, class Transcript, template <class> class... Relations> class Sumcheck {

  public:
    static constexpr size_t MAX_RELATION_LENGTH = std::max({ Relations<FF>::RELATION_LENGTH... });
    static constexpr size_t NUM_POLYNOMIALS = proof_system::honk::StandardArithmetization::NUM_POLYNOMIALS;

    std::array<FF, NUM_POLYNOMIALS> purported_evaluations;
    Transcript& transcript;
    const size_t multivariate_n;
    const size_t multivariate_d;
    SumcheckRound<FF, NUM_POLYNOMIALS, Relations...> round;

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
    * TODO(#224)(Cody): might want to just do C-style multidimensional array? for guaranteed adjacency?
    */
    std::array<std::vector<FF>, NUM_POLYNOMIALS> folded_polynomials;

    // prover instantiates sumcheck with circuit size and a prover transcript
    Sumcheck(size_t multivariate_n, ProverTranscript<FF>& transcript)
        : transcript(transcript)
        , multivariate_n(multivariate_n)
        , multivariate_d(numeric::get_msb(multivariate_n))
        , round(multivariate_n, std::tuple(Relations<FF>()...))
    {
        for (auto& polynomial : folded_polynomials) {
            polynomial.resize(multivariate_n >> 1);
        }
    };

    // verifier instantiates sumcheck with circuit size and a verifier transcript
    explicit Sumcheck(size_t multivariate_n, VerifierTranscript<FF>& transcript)
        : transcript(transcript)
        , multivariate_n(multivariate_n)
        , multivariate_d(numeric::get_msb(multivariate_n))
        , round(std::tuple(Relations<FF>()...)){};

    /**
     * @brief Compute univariate restriction place in transcript, generate challenge, fold,... repeat until final round,
     * then compute multivariate evaluations and place in transcript.
     *
     * @details
     */
    SumcheckOutput<FF> execute_prover(
        auto full_polynomials, const RelationParameters<FF>& relation_parameters) // pass by value, not by reference
    {
        auto [alpha, zeta] = transcript.get_challenges("Sumcheck:alpha", "Sumcheck:zeta");

        PowUnivariate<FF> pow_univariate(zeta);

        std::vector<FF> multivariate_challenge;
        multivariate_challenge.reserve(multivariate_d);

        // First round
        // This populates folded_polynomials.
        auto round_univariate = round.compute_univariate(full_polynomials, relation_parameters, pow_univariate, alpha);
        transcript.send_to_verifier("Sumcheck:univariate_0", round_univariate);
        FF round_challenge = transcript.get_challenge("Sumcheck:u_0");
        multivariate_challenge.emplace_back(round_challenge);
        fold(full_polynomials, multivariate_n, round_challenge);
        pow_univariate.partially_evaluate(round_challenge);
        round.round_size = round.round_size >> 1; // TODO(#224)(Cody): Maybe fold should do this and release memory?

        // All but final round
        // We operate on folded_polynomials in place.
        for (size_t round_idx = 1; round_idx < multivariate_d; round_idx++) {
            // Write the round univariate to the transcript
            round_univariate = round.compute_univariate(folded_polynomials, relation_parameters, pow_univariate, alpha);
            transcript.send_to_verifier("Sumcheck:univariate_" + std::to_string(round_idx), round_univariate);
            FF round_challenge = transcript.get_challenge("Sumcheck:u_" + std::to_string(round_idx));
            multivariate_challenge.emplace_back(round_challenge);
            fold(folded_polynomials, round.round_size, round_challenge);
            pow_univariate.partially_evaluate(round_challenge);
            round.round_size = round.round_size >> 1;
        }

        // Final round: Extract multivariate evaluations from folded_polynomials and add to transcript
        std::array<FF, NUM_POLYNOMIALS> multivariate_evaluations;
        for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
            multivariate_evaluations[i] = folded_polynomials[i][0];
        }
        transcript.send_to_verifier("Sumcheck:evaluations", multivariate_evaluations);

        return { multivariate_challenge, multivariate_evaluations };
    };

    /**
     * @brief Extract round univariate, check sum, generate challenge, compute next target sum..., repeat until final
     * round, then use purported evaluations to generate purported full Honk relation value and check against final
     * target sum.
     *
     * @details If verification fails, returns std::nullopt, otherwise returns SumcheckOutput
     */
    std::optional<SumcheckOutput<FF>> execute_verifier(const RelationParameters<FF>& relation_parameters)
    {
        bool verified(true);

        auto [alpha, zeta] = transcript.get_challenges("Sumcheck:alpha", "Sumcheck:zeta");

        PowUnivariate<FF> pow_univariate(zeta);
        // All but final round.
        // target_total_sum is initialized to zero then mutated in place.

        if (multivariate_d == 0) {
            throw_or_abort("Number of variables in multivariate is 0.");
        }

        std::vector<FF> multivariate_challenge;
        multivariate_challenge.reserve(multivariate_d);

        for (size_t round_idx = 0; round_idx < multivariate_d; round_idx++) {
            // Obtain the round univariate from the transcript
            std::string round_univariate_label = "Sumcheck:univariate_" + std::to_string(round_idx);
            auto round_univariate =
                transcript.template receive_from_prover<Univariate<FF, MAX_RELATION_LENGTH>>(round_univariate_label);

            bool checked = round.check_sum(round_univariate, pow_univariate);
            verified = verified && checked;
            FF round_challenge = transcript.get_challenge("Sumcheck:u_" + std::to_string(round_idx));
            multivariate_challenge.emplace_back(round_challenge);

            round.compute_next_target_sum(round_univariate, round_challenge, pow_univariate);
            pow_univariate.partially_evaluate(round_challenge);

            if (!verified) {
                return std::nullopt;
            }
        }

        // Final round
        auto purported_evaluations =
            transcript.template receive_from_prover<std::array<FF, NUM_POLYNOMIALS>>("Sumcheck:evaluations");

        FF full_honk_relation_purported_value = round.compute_full_honk_relation_purported_value(
            purported_evaluations, relation_parameters, pow_univariate, alpha);
        verified = verified && (full_honk_relation_purported_value == round.target_total_sum);
        if (!verified) {
            return std::nullopt;
        }

        return SumcheckOutput<FF>{ multivariate_challenge, purported_evaluations };
    };

    // TODO(#224)(Cody): Rename. fold is not descriptive, and it's already in use in the Gemini context.
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
            for (size_t i = 0; i < round_size; i += 2) {
                folded_polynomials[j][i >> 1] =
                    polynomials[j][i] + round_challenge * (polynomials[j][i + 1] - polynomials[j][i]);
            }
        }
    };
};
} // namespace proof_system::honk::sumcheck
