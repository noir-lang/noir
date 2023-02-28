#include "shplonk_single.hpp"
#include "../gemini/gemini.hpp"

#include <gtest/internal/gtest-internal.h>
#include <random>
#include <iterator>
#include <algorithm>

#include "../commitment_key.test.hpp"
#include "honk/pcs/claim.hpp"
#include "polynomials/polynomial.hpp"
namespace honk::pcs::shplonk {
template <class Params> class ShplonkTest : public CommitmentTest<Params> {};

TYPED_TEST_SUITE(ShplonkTest, CommitmentSchemeParams);

// Test of Shplonk prover/verifier using real Gemini claim
TYPED_TEST(ShplonkTest, GeminiShplonk)
{
    using Shplonk = SingleBatchOpeningScheme<TypeParam>;
    using Gemini = gemini::MultilinearReductionScheme<TypeParam>;
    using Fr = typename TypeParam::Fr;
    using Commitment = typename TypeParam::Commitment;
    using Polynomial = typename barretenberg::Polynomial<Fr>;

    const size_t n = 16;
    const size_t log_n = 4;

    using Transcript = transcript::StandardTranscript;
    auto transcript = std::make_shared<Transcript>(StandardHonk::create_manifest(0, log_n));
    transcript->mock_inputs_prior_to_challenge("rho");
    transcript->apply_fiat_shamir("rho");
    const Fr rho = Fr::serialize_from_buffer(transcript->get_challenge("rho").begin());

    const auto u = this->random_evaluation_point(log_n);
    auto poly = this->random_polynomial(n);
    const auto commitment = this->commit(poly);
    const auto eval = poly.evaluate_mle(u);

    // Collect multilinear polynomials evaluations, and commitments for input to prover/verifier
    std::vector<Fr> multilinear_evaluations = { eval };

    std::vector<Fr> rhos = Gemini::powers_of_rho(rho, multilinear_evaluations.size());

    // Compute batched multivariate evaluation
    Fr batched_evaluation = multilinear_evaluations[0] * rhos[0];

    Polynomial batched_unshifted(n);
    Polynomial batched_to_be_shifted(n);
    batched_unshifted.add_scaled(poly, rhos[0]);

    Commitment batched_commitment_unshifted = commitment * rhos[0];
    Commitment batched_commitment_to_be_shifted = Commitment::zero();

    auto gemini_prover_output =
        Gemini::reduce_prove(this->ck(), u, std::move(batched_unshifted), std::move(batched_to_be_shifted), transcript);

    const auto [prover_opening_pair, shplonk_prover_witness] = Shplonk::reduce_prove(
        this->ck(), gemini_prover_output.opening_pairs, gemini_prover_output.witnesses, transcript);

    this->verify_opening_pair(prover_opening_pair, shplonk_prover_witness);

    // Reconstruct a Gemini proof object consisting of
    // - d Fold poly evaluations a_0, ..., a_{d-1}
    // - (d-1) Fold polynomial commitments [Fold^(1)], ..., [Fold^(d-1)]
    auto gemini_proof = Gemini::reconstruct_proof_from_transcript(transcript, log_n);

    auto gemini_verifier_claim = Gemini::reduce_verify(u,
                                                       batched_evaluation,
                                                       batched_commitment_unshifted,
                                                       batched_commitment_to_be_shifted,
                                                       gemini_proof,
                                                       transcript);

    // Reconstruct the Shplonk Proof (commitment [Q]) from the transcript
    auto shplonk_proof = transcript->get_group_element("Q");

    const auto verifier_claim = Shplonk::reduce_verify(gemini_verifier_claim, shplonk_proof, transcript);

    this->verify_opening_claim(verifier_claim, shplonk_prover_witness);
}
} // namespace honk::pcs::shplonk