
#include "kzg.hpp"
#include "../shplonk/shplonk_single.hpp"
#include "../gemini/gemini.hpp"

#include "../commitment_key.test.hpp"
#include "honk/pcs/claim.hpp"
#include "honk/pcs/commitment_key.hpp"
#include "polynomials/polynomial.hpp"

#include <ecc/curves/bn254/g1.hpp>

#include <gtest/gtest.h>
#include <vector>

namespace honk::pcs::kzg {

template <class Params> class BilinearAccumulationTest : public CommitmentTest<Params> {
  public:
    using Fr = typename Params::Fr;
    using Commitment = typename Params::Commitment;
    using Polynomial = barretenberg::Polynomial<Fr>;
};

TYPED_TEST_SUITE(BilinearAccumulationTest, CommitmentSchemeParams);

TYPED_TEST(BilinearAccumulationTest, single)
{
    const size_t n = 16;
    const size_t log_n = 4;

    using KZG = UnivariateOpeningScheme<TypeParam>;
    using Fr = typename TypeParam::Fr;

    // Instantiate a transcript from the real Honk manifest, then mock the inputs prior to Gemini.
    using Transcript = transcript::StandardTranscript;
    auto transcript = std::make_shared<Transcript>(StandardHonk::create_unrolled_manifest(0, log_n));
    transcript->mock_inputs_prior_to_challenge("z");

    auto witness = this->random_polynomial(n);
    auto commitment = this->commit(witness);
    auto query = Fr::random_element();
    auto evaluation = witness.evaluate(query);
    auto opening_pair = OpeningPair<TypeParam>{ query, evaluation };

    KZG::reduce_prove(this->ck(), opening_pair, witness, transcript);

    // Reconstruct the KZG Proof (commitment [W]) from the transcript
    auto kzg_proof = transcript->get_group_element("W");

    auto opening_claim = OpeningClaim<TypeParam>{ opening_pair, commitment };

    auto kzg_claim = KZG::reduce_verify(opening_claim, kzg_proof);

    bool verified = kzg_claim.verify(this->vk());

    EXPECT_EQ(verified, true);
}

/**
 * @brief Test full PCS protocol: Gemini, Shplonk, KZG and pairing check
 * @details Demonstrates the full PCS protocol as it is used in the construction and verification
 * of a single Honk proof. (Expository comments included throughout).
 *
 */
TYPED_TEST(BilinearAccumulationTest, GeminiShplonkKzgWithShift)
{
    using Transcript = transcript::StandardTranscript;
    using Shplonk = shplonk::SingleBatchOpeningScheme<TypeParam>;
    using Gemini = gemini::MultilinearReductionScheme<TypeParam>;
    using KZG = UnivariateOpeningScheme<TypeParam>;
    using Fr = typename TypeParam::Fr;
    using Commitment = typename TypeParam::Commitment;
    using Polynomial = typename barretenberg::Polynomial<Fr>;

    const size_t n = 16;
    const size_t log_n = 4;

    // Instantiate a transcript from the real Honk manifest, then mock the inputs prior to Gemini.
    auto transcript = std::make_shared<Transcript>(StandardHonk::create_unrolled_manifest(0, log_n));
    transcript->mock_inputs_prior_to_challenge("rho");
    transcript->apply_fiat_shamir("rho");
    const Fr rho = Fr::serialize_from_buffer(transcript->get_challenge("rho").begin());

    // Generate multilinear polynomials, their commitments (genuine and mocked) and evaluations (genuine) at a random
    // point.
    const auto mle_opening_point = this->random_evaluation_point(log_n); // sometimes denoted 'u'
    auto poly1 = this->random_polynomial(n);
    auto poly2 = this->random_polynomial(n);
    poly2[0] = Params::Fr::zero(); // this property is required of polynomials whose shift is used

    auto commitment1 = this->commit(poly1);
    auto commitment2 = this->commit(poly2);

    auto eval1 = poly1.evaluate_mle(mle_opening_point);
    auto eval2 = poly2.evaluate_mle(mle_opening_point);
    auto eval2_shift = poly2.evaluate_mle(mle_opening_point, true);

    // Collect multilinear evaluations for input to prover
    std::vector<Fr> multilinear_evaluations = { eval1, eval2, eval2_shift };

    std::vector<Fr> rhos = Gemini::powers_of_rho(rho, multilinear_evaluations.size());

    // Compute batched multivariate evaluation
    Fr batched_evaluation = Fr::zero();
    for (size_t i = 0; i < rhos.size(); ++i) {
        batched_evaluation += multilinear_evaluations[i] * rhos[i];
    }

    // Compute batched polynomials
    Polynomial batched_unshifted(n);
    Polynomial batched_to_be_shifted(n);
    batched_unshifted.add_scaled(poly1, rhos[0]);
    batched_unshifted.add_scaled(poly2, rhos[1]);
    batched_to_be_shifted.add_scaled(poly2, rhos[2]);

    // Compute batched commitments
    Commitment batched_commitment_unshifted = Commitment::zero();
    Commitment batched_commitment_to_be_shifted = Commitment::zero();
    batched_commitment_unshifted = commitment1 * rhos[0] + commitment2 * rhos[1];
    batched_commitment_to_be_shifted = commitment2 * rhos[2];

    // Run the full prover PCS protocol:

    // Gemini prover output:
    // - opening pairs: d+1 pairs (r, a_0_pos) and (-r^{2^l}, a_l), l = 0:d-1
    // - witness: the d+1 polynomials Fold_{r}^(0), Fold_{-r}^(0), Fold^(l), l = 1:d-1
    auto gemini_prover_output = Gemini::reduce_prove(
        this->ck(), mle_opening_point, std::move(batched_unshifted), std::move(batched_to_be_shifted), transcript);

    // Shplonk prover output:
    // - opening pair: (z_challenge, 0)
    // - witness: polynomial Q - Q_z
    auto shplonk_prover_output = Shplonk::reduce_prove(
        this->ck(), gemini_prover_output.opening_pairs, gemini_prover_output.witnesses, transcript);

    // KZG prover:
    // - Adds commitment [W] to transcript
    KZG::reduce_prove(this->ck(), shplonk_prover_output.opening_pair, shplonk_prover_output.witness, transcript);

    // Run the full verifier PCS protocol with genuine opening claims (genuine commitment, genuine evaluation)

    // Construct a Gemini proof object consisting of
    // - d Fold poly evaluations a_0, ..., a_{d-1}
    // - (d-1) Fold polynomial commitments [Fold^(1)], ..., [Fold^(d-1)]
    auto gemini_proof = Gemini::reconstruct_proof_from_transcript(transcript, log_n);

    // Reconstruct the Shplonk Proof (commitment [Q]) from the transcript
    auto shplonk_proof = transcript->get_group_element("Q");

    // Reconstruct the KZG Proof (commitment [W]) from the transcript
    auto kzg_proof = transcript->get_group_element("W");

    // Gemini verifier output:
    // - claim: d+1 commitments to Fold_{r}^(0), Fold_{-r}^(0), Fold^(l), d+1 evaluations a_0_pos, a_l, l = 0:d-1
    auto gemini_verifier_claim = Gemini::reduce_verify(mle_opening_point,
                                                       batched_evaluation,
                                                       batched_commitment_unshifted,
                                                       batched_commitment_to_be_shifted,
                                                       gemini_proof,
                                                       transcript);

    // Shplonk verifier claim: commitment [Q] - [Q_z], opening point (z_challenge, 0)
    const auto shplonk_verifier_claim = Shplonk::reduce_verify(gemini_verifier_claim, shplonk_proof, transcript);

    // KZG verifier:
    // aggregates inputs [Q] - [Q_z] and [W] into an 'accumulator' (can perform pairing check on result)
    auto kzg_claim = KZG::reduce_verify(shplonk_verifier_claim, kzg_proof);

    // Final pairing check: e([Q] - [Q_z] + z[W], [1]_2) = e([W], [x]_2)
    bool verified = kzg_claim.verify(this->vk());

    EXPECT_EQ(verified, true);
}

} // namespace honk::pcs::kzg