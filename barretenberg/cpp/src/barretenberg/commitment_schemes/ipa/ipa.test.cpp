#include "ipa.hpp"
#include "../gemini/gemini.hpp"
#include "../shplonk/shplonk.hpp"
#include "barretenberg/commitment_schemes/commitment_key.test.hpp"
#include "barretenberg/common/mem.hpp"
#include "barretenberg/ecc/curves/bn254/fq12.hpp"
#include "barretenberg/ecc/curves/types.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/polynomials/polynomial_arithmetic.hpp"
#include <gtest/gtest.h>

using namespace bb;
using namespace bb::honk;
using namespace bb::honk::pcs;
using namespace bb::honk::pcs::ipa;

using Curve = curve::Grumpkin;

class IPATest : public CommitmentTest<Curve> {
  public:
    using Fr = typename Curve::ScalarField;
    using GroupElement = typename Curve::Element;
    using CK = CommitmentKey<Curve>;
    using VK = VerifierCommitmentKey<Curve>;
    using Polynomial = bb::Polynomial<Fr>;
};

TEST_F(IPATest, CommitOnManyZeroCoeffPolyWorks)
{
    constexpr size_t n = 4;
    Polynomial p(n);
    for (size_t i = 0; i < n - 1; i++) {
        p[i] = Fr::zero();
    }
    p[3] = Fr::one();
    GroupElement commitment = this->commit(p);
    auto srs_elements = this->ck()->srs->get_monomial_points();
    GroupElement expected = srs_elements[0] * p[0];
    // The SRS stored in the commitment key is the result after applying the pippenger point table so the
    // values at odd indices contain the point {srs[i-1].x * beta, srs[i-1].y}, where beta is the endomorphism
    // G_vec_local should use only the original SRS thus we extract only the even indices.
    for (size_t i = 2; i < 2 * n; i += 2) {
        expected += srs_elements[i] * p[i >> 1];
    }
    EXPECT_EQ(expected.normalize(), commitment.normalize());
}

TEST_F(IPATest, Commit)
{
    constexpr size_t n = 128;
    auto poly = this->random_polynomial(n);
    GroupElement commitment = this->commit(poly);
    auto srs_elements = this->ck()->srs->get_monomial_points();
    GroupElement expected = srs_elements[0] * poly[0];
    // The SRS stored in the commitment key is the result after applying the pippenger point table so the
    // values at odd indices contain the point {srs[i-1].x * beta, srs[i-1].y}, where beta is the endomorphism
    // G_vec_local should use only the original SRS thus we extract only the even indices.
    for (size_t i = 2; i < 2 * n; i += 2) {
        expected += srs_elements[i] * poly[i >> 1];
    }
    EXPECT_EQ(expected.normalize(), commitment.normalize());
}

TEST_F(IPATest, Open)
{
    using IPA = IPA<Curve>;
    // generate a random polynomial, degree needs to be a power of two
    size_t n = 128;
    auto poly = this->random_polynomial(n);
    auto [x, eval] = this->random_eval(poly);
    auto commitment = this->commit(poly);
    const OpeningPair<Curve> opening_pair = { x, eval };
    const OpeningClaim<Curve> opening_claim{ opening_pair, commitment };

    // initialize empty prover transcript
    auto prover_transcript = std::make_shared<BaseTranscript>();
    IPA::compute_opening_proof(this->ck(), opening_pair, poly, prover_transcript);

    // initialize verifier transcript from proof data
    auto verifier_transcript = std::make_shared<BaseTranscript>(prover_transcript->proof_data);

    auto result = IPA::verify(this->vk(), opening_claim, verifier_transcript);
    EXPECT_TRUE(result);

    EXPECT_EQ(prover_transcript->get_manifest(), verifier_transcript->get_manifest());
}

TEST_F(IPATest, GeminiShplonkIPAWithShift)
{
    using IPA = IPA<Curve>;
    using ShplonkProver = shplonk::ShplonkProver_<Curve>;
    using ShplonkVerifier = shplonk::ShplonkVerifier_<Curve>;
    using GeminiProver = gemini::GeminiProver_<Curve>;
    using GeminiVerifier = gemini::GeminiVerifier_<Curve>;

    const size_t n = 8;
    const size_t log_n = 3;

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

    std::vector<Fr> multilinear_evaluations = { eval1, eval2, eval2_shift };

    std::vector<Fr> rhos = gemini::powers_of_rho(rho, multilinear_evaluations.size());

    Fr batched_evaluation = Fr::zero();
    for (size_t i = 0; i < rhos.size(); ++i) {
        batched_evaluation += multilinear_evaluations[i] * rhos[i];
    }

    Polynomial batched_unshifted(n);
    Polynomial batched_to_be_shifted(n);
    batched_unshifted.add_scaled(poly1, rhos[0]);
    batched_unshifted.add_scaled(poly2, rhos[1]);
    batched_to_be_shifted.add_scaled(poly2, rhos[2]);

    GroupElement batched_commitment_unshifted = GroupElement::zero();
    GroupElement batched_commitment_to_be_shifted = GroupElement::zero();
    batched_commitment_unshifted = commitment1 * rhos[0] + commitment2 * rhos[1];
    batched_commitment_to_be_shifted = commitment2 * rhos[2];

    auto prover_transcript = BaseTranscript::prover_init_empty();

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

    const Fr nu_challenge = prover_transcript->get_challenge("Shplonk:nu");
    auto batched_quotient_Q =
        ShplonkProver::compute_batched_quotient(gemini_opening_pairs, gemini_witnesses, nu_challenge);
    prover_transcript->send_to_verifier("Shplonk:Q", this->ck()->commit(batched_quotient_Q));

    const Fr z_challenge = prover_transcript->get_challenge("Shplonk:z");
    const auto [shplonk_opening_pair, shplonk_witness] = ShplonkProver::compute_partially_evaluated_batched_quotient(
        gemini_opening_pairs, gemini_witnesses, std::move(batched_quotient_Q), nu_challenge, z_challenge);

    IPA::compute_opening_proof(this->ck(), shplonk_opening_pair, shplonk_witness, prover_transcript);

    auto verifier_transcript = BaseTranscript::verifier_init_empty(prover_transcript);

    auto gemini_verifier_claim = GeminiVerifier::reduce_verification(mle_opening_point,
                                                                     batched_evaluation,
                                                                     batched_commitment_unshifted,
                                                                     batched_commitment_to_be_shifted,
                                                                     verifier_transcript);

    const auto shplonk_verifier_claim =
        ShplonkVerifier::reduce_verification(this->vk(), gemini_verifier_claim, verifier_transcript);
    bool verified = IPA::verify(this->vk(), shplonk_verifier_claim, verifier_transcript);

    EXPECT_EQ(verified, true);
}
