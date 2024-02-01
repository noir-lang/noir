#pragma once
#include "barretenberg/proof_system/library/grand_product_delta.hpp"
#include "barretenberg/sumcheck/instance/prover_instance.hpp"
#include "barretenberg/sumcheck/sumcheck_output.hpp"
#include "barretenberg/transcript/transcript.hpp"
#include "sumcheck_round.hpp"

namespace bb {

template <typename Flavor> class SumcheckProver {

  public:
    using FF = typename Flavor::FF;
    using ProverPolynomials = typename Flavor::ProverPolynomials;
    using PartiallyEvaluatedMultivariates = typename Flavor::PartiallyEvaluatedMultivariates;
    using ClaimedEvaluations = typename Flavor::AllValues;
    using Transcript = typename Flavor::Transcript;
    using Instance = ProverInstance_<Flavor>;
    using RelationSeparator = typename Flavor::RelationSeparator;

    const size_t multivariate_n;
    const size_t multivariate_d;

    std::shared_ptr<Transcript> transcript;
    SumcheckProverRound<Flavor> round;

    /**
    *
    * @brief (partially_evaluated_polynomials) Suppose the Honk polynomials (multilinear in d variables) are called P_1,
    ..., P_N.
    * At initialization,
    * we think of these as lying in a two-dimensional array, where each column records the value of one P_i on H^d.
    * After the first round, the array will be updated (partially evaluated), so that the first n/2 rows will represent
    the
    * evaluations P_i(u0, X1, ..., X_{d-1}) as a low-degree extension on H^{d-1}. In reality, we elude copying all
    * of the polynomial-defining data by only populating partially_evaluated_polynomials after the first round. I.e.:

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
            Each group consists of N edges |, and our construction of univariates and partial evaluation
                                        *
            operations naturally operate on these groups of edges

    *
    * NOTE: With ~40 columns, prob only want to allocate 256 EdgeGroup's at once to keep stack under 1MB?
    * TODO(#224)(Cody): might want to just do C-style multidimensional array? for guaranteed adjacency?
    */
    PartiallyEvaluatedMultivariates partially_evaluated_polynomials;

    // prover instantiates sumcheck with circuit size and a prover transcript
    SumcheckProver(size_t multivariate_n, const std::shared_ptr<Transcript>& transcript)
        : multivariate_n(multivariate_n)
        , multivariate_d(numeric::get_msb(multivariate_n))
        , transcript(transcript)
        , round(multivariate_n)
        , partially_evaluated_polynomials(multivariate_n){};

    // WORKTODO delete this
    /**
     * @brief Compute univariate restriction place in transcript, generate challenge, partially evaluate,... repeat
     * until final round, then compute multivariate evaluations and place in transcript.
     */
    SumcheckOutput<Flavor> prove(std::shared_ptr<Instance> instance)
    {
        return prove(
            instance->prover_polynomials, instance->relation_parameters, instance->alphas, instance->gate_challenges);
    };

    /**
     * @brief Compute univariate restriction place in transcript, generate challenge, partially evaluate,... repeat
     * until final round, then compute multivariate evaluations and place in transcript.
     *
     * @details
     */
    SumcheckOutput<Flavor> prove(ProverPolynomials& full_polynomials,
                                 const bb::RelationParameters<FF>& relation_parameters,
                                 const RelationSeparator alpha,
                                 const std::vector<FF>& gate_challenges)
    {

        bb::PowPolynomial<FF> pow_univariate(gate_challenges);
        pow_univariate.compute_values();

        std::vector<FF> multivariate_challenge;
        multivariate_challenge.reserve(multivariate_d);

        // First round
        // This populates partially_evaluated_polynomials.
        auto round_univariate = round.compute_univariate(full_polynomials, relation_parameters, pow_univariate, alpha);
        transcript->send_to_verifier("Sumcheck:univariate_0", round_univariate);
        FF round_challenge = transcript->get_challenge("Sumcheck:u_0");
        multivariate_challenge.emplace_back(round_challenge);
        partially_evaluate(full_polynomials, multivariate_n, round_challenge);
        pow_univariate.partially_evaluate(round_challenge);
        round.round_size = round.round_size >> 1; // TODO(#224)(Cody): Maybe partially_evaluate should do this and
                                                  // release memory?        // All but final round
        // We operate on partially_evaluated_polynomials in place.
        for (size_t round_idx = 1; round_idx < multivariate_d; round_idx++) {
            // Write the round univariate to the transcript
            round_univariate =
                round.compute_univariate(partially_evaluated_polynomials, relation_parameters, pow_univariate, alpha);
            transcript->send_to_verifier("Sumcheck:univariate_" + std::to_string(round_idx), round_univariate);
            FF round_challenge = transcript->get_challenge("Sumcheck:u_" + std::to_string(round_idx));
            multivariate_challenge.emplace_back(round_challenge);
            partially_evaluate(partially_evaluated_polynomials, round.round_size, round_challenge);
            pow_univariate.partially_evaluate(round_challenge);
            round.round_size = round.round_size >> 1;
        }

        // Final round: Extract multivariate evaluations from partially_evaluated_polynomials and add to transcript
        ClaimedEvaluations multivariate_evaluations;
        for (auto [eval, poly] :
             zip_view(multivariate_evaluations.get_all(), partially_evaluated_polynomials.get_all())) {
            eval = poly[0];
        }
        transcript->send_to_verifier("Sumcheck:evaluations", multivariate_evaluations);

        return { multivariate_challenge, multivariate_evaluations };
    };

    /**
     * @brief Evaluate at the round challenge and prepare class for next round.
     * Illustration of layout in example of first round when d==3 (showing just one Honk polynomial,
     * i.e., what happens in just one column of our two-dimensional array):
     *
     * groups    vertex terms              collected vertex terms               groups after partial evaluation
     *     g0 -- v0 (1-X0)(1-X1)(1-X2) --- (v0(1-X0) + v1 X0) (1-X1)(1-X2) ---- (v0(1-u0) + v1 u0) (1-X1)(1-X2)
     *        \- v1   X0  (1-X1)(1-X2) --/                                  --- (v2(1-u0) + v3 u0)   X1  (1-X2)
     *     g1 -- v2 (1-X0)  X1  (1-X2) --- (v2(1-X0) + v3 X0)   X1  (1-X2)-/ -- (v4(1-u0) + v5 u0) (1-X1)  X2
     *        \- v3   X0    X1  (1-X2) --/                                  / - (v6(1-u0) + v7 u0)   X1    X2
     *     g2 -- v4 (1-X0)(1-X1)  X2   --- (v4(1-X0) + v5 X0) (1-X1)  X2  -/ /
     *        \- v5   X0  (1-X1)  X2   --/                                  /
     *     g3 -- v6 (1-X0)  X1    X2   --- (v6(1-X0) + v7 X0)   X1    X2  -/
     *        \- v7   X0    X1    X2   --/
     */
    void partially_evaluate(auto& polynomials, size_t round_size, FF round_challenge)
    {
        auto pep_view = partially_evaluated_polynomials.get_all();
        auto poly_view = polynomials.get_all();
        // after the first round, operate in place on partially_evaluated_polynomials
        parallel_for(poly_view.size(), [&](size_t j) {
            for (size_t i = 0; i < round_size; i += 2) {
                pep_view[j][i >> 1] = poly_view[j][i] + round_challenge * (poly_view[j][i + 1] - poly_view[j][i]);
            }
        });
    };
    /**
     * @brief Evaluate at the round challenge and prepare class for next round.
     * Specialization for array, see generic version above.
     */
    template <typename PolynomialT, std::size_t N>
    void partially_evaluate(std::array<PolynomialT, N>& polynomials, size_t round_size, FF round_challenge)
    {
        auto pep_view = partially_evaluated_polynomials.get_all();
        // after the first round, operate in place on partially_evaluated_polynomials
        parallel_for(polynomials.size(), [&](size_t j) {
            for (size_t i = 0; i < round_size; i += 2) {
                pep_view[j][i >> 1] = polynomials[j][i] + round_challenge * (polynomials[j][i + 1] - polynomials[j][i]);
            }
        });
    };
};

template <typename Flavor> class SumcheckVerifier {

  public:
    using Utils = bb::RelationUtils<Flavor>;
    using FF = typename Flavor::FF;
    using ClaimedEvaluations = typename Flavor::AllValues;
    using Transcript = typename Flavor::Transcript;
    using RelationSeparator = typename Flavor::RelationSeparator;

    static constexpr size_t BATCHED_RELATION_PARTIAL_LENGTH = Flavor::BATCHED_RELATION_PARTIAL_LENGTH;
    static constexpr size_t NUM_POLYNOMIALS = Flavor::NUM_ALL_ENTITIES;

    const size_t multivariate_d;
    std::shared_ptr<Transcript> transcript;
    SumcheckVerifierRound<Flavor> round;

    // Verifier instantiates sumcheck with circuit size, optionally a different target sum than 0 can be specified.
    explicit SumcheckVerifier(size_t multivariate_d, std::shared_ptr<Transcript> transcript, FF target_sum = 0)
        : multivariate_d(multivariate_d)
        , transcript(transcript)
        , round(target_sum){};

    /**
     * @brief Extract round univariate, check sum, generate challenge, compute next target sum..., repeat until
     * final round, then use purported evaluations to generate purported full Honk relation value and check against
     * final target sum.
     *
     * @details If verification fails, returns std::nullopt, otherwise returns SumcheckOutput
     * @param relation_parameters
     * @param transcript
     */
    SumcheckOutput<Flavor> verify(const bb::RelationParameters<FF>& relation_parameters,
                                  RelationSeparator alpha,
                                  const std::vector<FF>& gate_challenges)
    {
        bool verified(true);

        bb::PowPolynomial<FF> pow_univariate(gate_challenges);
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
                transcript->template receive_from_prover<bb::Univariate<FF, BATCHED_RELATION_PARTIAL_LENGTH>>(
                    round_univariate_label);

            bool checked = round.check_sum(round_univariate);
            verified = verified && checked;
            FF round_challenge = transcript->get_challenge("Sumcheck:u_" + std::to_string(round_idx));
            multivariate_challenge.emplace_back(round_challenge);

            round.compute_next_target_sum(round_univariate, round_challenge);
            pow_univariate.partially_evaluate(round_challenge);
        }

        // Final round
        ClaimedEvaluations purported_evaluations;
        auto transcript_evaluations =
            transcript->template receive_from_prover<std::array<FF, NUM_POLYNOMIALS>>("Sumcheck:evaluations");
        for (auto [eval, transcript_eval] : zip_view(purported_evaluations.get_all(), transcript_evaluations)) {
            eval = transcript_eval;
        }

        FF full_honk_relation_purported_value = round.compute_full_honk_relation_purported_value(
            purported_evaluations, relation_parameters, pow_univariate, alpha);

        bool checked = false;
        if constexpr (IsRecursiveFlavor<Flavor>) {
            checked = (full_honk_relation_purported_value == round.target_total_sum).get_value();
        } else {
            checked = (full_honk_relation_purported_value == round.target_total_sum);
        }
        verified = verified && checked;

        return SumcheckOutput<Flavor>{ multivariate_challenge, purported_evaluations, verified };
    };
};
} // namespace bb
