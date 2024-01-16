
#include "kzg.hpp"
#include "../gemini/gemini.hpp"
#include "../shplonk/shplonk.hpp"

#include "../commitment_key.test.hpp"
#include "barretenberg/commitment_schemes/claim.hpp"
#include "barretenberg/polynomials/polynomial.hpp"

#include "barretenberg/ecc/curves/bn254/g1.hpp"

#include <gtest/gtest.h>
#include <vector>

namespace proof_system::honk::pcs::kzg {

template <class Curve> class KZGTest : public CommitmentTest<Curve> {
  public:
    using Fr = typename Curve::ScalarField;
    using Commitment = typename Curve::AffineElement;
    using GroupElement = typename Curve::Element;
    using Polynomial = bb::Polynomial<Fr>;
};

TYPED_TEST_SUITE(KZGTest, CommitmentSchemeParams);

TYPED_TEST(KZGTest, single)
{
    const size_t n = 16;

    using KZG = KZG<TypeParam>;
    using Fr = typename TypeParam::ScalarField;

    auto witness = this->random_polynomial(n);
    bb::g1::element commitment = this->commit(witness);

    auto challenge = Fr::random_element();
    auto evaluation = witness.evaluate(challenge);
    auto opening_pair = OpeningPair<TypeParam>{ challenge, evaluation };
    auto opening_claim = OpeningClaim<TypeParam>{ opening_pair, commitment };

    auto prover_transcript = BaseTranscript::prover_init_empty();

    KZG::compute_opening_proof(this->ck(), opening_pair, witness, prover_transcript);

    auto verifier_transcript = BaseTranscript::verifier_init_empty(prover_transcript);
    bool verified = KZG::verify(this->vk(), opening_claim, verifier_transcript);

    EXPECT_EQ(verified, true);
}

/**
 * @brief Test full PCS protocol: Gemini, Shplonk, KZG and pairing check
 * @details Demonstrates the full PCS protocol as it is used in the construction and verification
 * of a single Honk proof. (Expository comments included throughout).
 *
 */
TYPED_TEST(KZGTest, GeminiShplonkKzgWithShift)
{
    using ShplonkProver = shplonk::ShplonkProver_<TypeParam>;
    using ShplonkVerifier = shplonk::ShplonkVerifier_<TypeParam>;
    using GeminiProver = gemini::GeminiProver_<TypeParam>;
    using GeminiVerifier = gemini::GeminiVerifier_<TypeParam>;
    using KZG = KZG<TypeParam>;
    using Fr = typename TypeParam::ScalarField;
    using GroupElement = typename TypeParam::Element;
    using Polynomial = typename bb::Polynomial<Fr>;

    const size_t n = 16;
    const size_t log_n = 4;

    Fr rho = Fr::random_element();

    // Generate multilinear polynomials, their commitments (genuine and mocked) and evaluations (genuine) at a random
    // point.
    const auto mle_opening_point = this->random_evaluation_point(log_n); // sometimes denoted 'u'
    auto poly1 = this->random_polynomial(n);
    auto poly2 = this->random_polynomial(n);
    poly2[0] = Fr::zero(); // this property is required of polynomials whose shift is used

    GroupElement commitment1 = this->commit(poly1);
    GroupElement commitment2 = this->commit(poly2);

    auto eval1 = poly1.evaluate_mle(mle_opening_point);
    auto eval2 = poly2.evaluate_mle(mle_opening_point);
    auto eval2_shift = poly2.evaluate_mle(mle_opening_point, true);

    // Collect multilinear evaluations for input to prover
    std::vector<Fr> multilinear_evaluations = { eval1, eval2, eval2_shift };

    std::vector<Fr> rhos = gemini::powers_of_rho(rho, multilinear_evaluations.size());

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
    GroupElement batched_commitment_unshifted = GroupElement::zero();
    GroupElement batched_commitment_to_be_shifted = GroupElement::zero();
    batched_commitment_unshifted = commitment1 * rhos[0] + commitment2 * rhos[1];
    batched_commitment_to_be_shifted = commitment2 * rhos[2];

    auto prover_transcript = BaseTranscript::prover_init_empty();

    // Run the full prover PCS protocol:

    // Compute:
    // - (d+1) opening pairs: {r, \hat{a}_0}, {-r^{2^i}, a_i}, i = 0, ..., d-1
    // - (d+1) Fold polynomials Fold_{r}^(0), Fold_{-r}^(0), and Fold^(i), i = 0, ..., d-1
    auto gemini_polynomials = GeminiProver::compute_gemini_polynomials(
        mle_opening_point, std::move(batched_unshifted), std::move(batched_to_be_shifted));

    for (size_t l = 0; l < log_n - 1; ++l) {
        std::string label = "FOLD_" + std::to_string(l + 1);
        auto commitment = this->ck()->commit(gemini_polynomials[l + 2]);
        prover_transcript->send_to_verifier(label, commitment);
    }

    const Fr r_challenge = prover_transcript->get_challenge("Gemini:r");

    const auto [gemini_opening_pairs, gemini_witnesses] = GeminiProver::compute_fold_polynomial_evaluations(
        mle_opening_point, std::move(gemini_polynomials), r_challenge);

    for (size_t l = 0; l < log_n; ++l) {
        std::string label = "Gemini:a_" + std::to_string(l);
        const auto& evaluation = gemini_opening_pairs[l + 1].evaluation;
        prover_transcript->send_to_verifier(label, evaluation);
    }

    // Shplonk prover output:
    // - opening pair: (z_challenge, 0)
    // - witness: polynomial Q - Q_z
    const Fr nu_challenge = prover_transcript->get_challenge("Shplonk:nu");
    auto batched_quotient_Q =
        ShplonkProver::compute_batched_quotient(gemini_opening_pairs, gemini_witnesses, nu_challenge);
    prover_transcript->send_to_verifier("Shplonk:Q", this->ck()->commit(batched_quotient_Q));

    const Fr z_challenge = prover_transcript->get_challenge("Shplonk:z");
    const auto [shplonk_opening_pair, shplonk_witness] = ShplonkProver::compute_partially_evaluated_batched_quotient(
        gemini_opening_pairs, gemini_witnesses, std::move(batched_quotient_Q), nu_challenge, z_challenge);

    // KZG prover:
    // - Adds commitment [W] to transcript
    KZG::compute_opening_proof(this->ck(), shplonk_opening_pair, shplonk_witness, prover_transcript);

    // Run the full verifier PCS protocol with genuine opening claims (genuine commitment, genuine evaluation)

    auto verifier_transcript = BaseTranscript::verifier_init_empty(prover_transcript);

    // Gemini verifier output:
    // - claim: d+1 commitments to Fold_{r}^(0), Fold_{-r}^(0), Fold^(l), d+1 evaluations a_0_pos, a_l, l = 0:d-1
    auto gemini_verifier_claim = GeminiVerifier::reduce_verification(mle_opening_point,
                                                                     batched_evaluation,
                                                                     batched_commitment_unshifted,
                                                                     batched_commitment_to_be_shifted,
                                                                     verifier_transcript);

    // Shplonk verifier claim: commitment [Q] - [Q_z], opening point (z_challenge, 0)
    const auto shplonk_verifier_claim =
        ShplonkVerifier::reduce_verification(this->vk(), gemini_verifier_claim, verifier_transcript);

    // KZG verifier:
    // aggregates inputs [Q] - [Q_z] and [W] into an 'accumulator' (can perform pairing check on result)
    bool verified = KZG::verify(this->vk(), shplonk_verifier_claim, verifier_transcript);

    // Final pairing check: e([Q] - [Q_z] + z[W], [1]_2) = e([W], [x]_2)

    EXPECT_EQ(verified, true);
}

} // namespace proof_system::honk::pcs::kzg
