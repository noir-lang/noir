
#include "../gemini/gemini.hpp"
#include "../shplonk/shplonk.hpp"
#include "./mock_transcript.hpp"
#include "barretenberg/commitment_schemes/commitment_key.test.hpp"
#include "barretenberg/common/mem.hpp"
#include "barretenberg/ecc/curves/bn254/fq12.hpp"
#include "barretenberg/ecc/curves/types.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/polynomials/polynomial_arithmetic.hpp"
#include <gtest/gtest.h>
#include <utility>

using namespace bb;

namespace {
using Curve = curve::Grumpkin;

class IPATest : public CommitmentTest<Curve> {
  public:
    using Fr = typename Curve::ScalarField;
    using GroupElement = typename Curve::Element;
    using CK = CommitmentKey<Curve>;
    using VK = VerifierCommitmentKey<Curve>;
    using Polynomial = bb::Polynomial<Fr>;
};
} // namespace

#define IPA_TEST
#include "ipa.hpp"

TEST_F(IPATest, CommitOnManyZeroCoeffPolyWorks)
{
    constexpr size_t n = 4;
    Polynomial p(n);
    for (size_t i = 0; i < n - 1; i++) {
        p[i] = Fr::zero();
    }
    p[3] = Fr::one();
    GroupElement commitment = this->commit(p);
    auto* srs_elements = this->ck()->srs->get_monomial_points();
    GroupElement expected = srs_elements[0] * p[0];
    // The SRS stored in the commitment key is the result after applying the pippenger point table so the
    // values at odd indices contain the point {srs[i-1].x * beta, srs[i-1].y}, where beta is the endomorphism
    // G_vec_local should use only the original SRS thus we extract only the even indices.
    for (size_t i = 2; i < 2 * n; i += 2) {
        expected += srs_elements[i] * p[i >> 1];
    }
    EXPECT_EQ(expected.normalize(), commitment.normalize());
}

// This test checks that we can correctly open a zero polynomial. Since we often have point at infinity troubles, it
// detects those.
TEST_F(IPATest, OpenZeroPolynomial)
{
    using IPA = IPA<Curve>;
    constexpr size_t n = 4;
    Polynomial poly(n);
    // Commit to a zero polynomial
    GroupElement commitment = this->commit(poly);
    EXPECT_TRUE(commitment.is_point_at_infinity());

    auto [x, eval] = this->random_eval(poly);
    EXPECT_EQ(eval, Fr::zero());
    const OpeningPair<Curve> opening_pair = { x, eval };
    const OpeningClaim<Curve> opening_claim{ opening_pair, commitment };

    // initialize empty prover transcript
    auto prover_transcript = std::make_shared<NativeTranscript>();
    IPA::compute_opening_proof(this->ck(), opening_pair, poly, prover_transcript);

    // initialize verifier transcript from proof data
    auto verifier_transcript = std::make_shared<NativeTranscript>(prover_transcript->proof_data);

    auto result = IPA::reduce_verify(this->vk(), opening_claim, verifier_transcript);
    EXPECT_TRUE(result);
}

// This test makes sure that even if the whole vector \vec{b} generated from the x, at which we open the polynomial, is
// zero, IPA behaves
TEST_F(IPATest, OpenAtZero)
{
    using IPA = IPA<Curve>;
    // generate a random polynomial, degree needs to be a power of two
    size_t n = 128;
    auto poly = this->random_polynomial(n);
    Fr x = Fr::zero();
    auto eval = poly.evaluate(x);
    auto commitment = this->commit(poly);
    const OpeningPair<Curve> opening_pair = { x, eval };
    const OpeningClaim<Curve> opening_claim{ opening_pair, commitment };

    // initialize empty prover transcript
    auto prover_transcript = std::make_shared<NativeTranscript>();
    IPA::compute_opening_proof(this->ck(), opening_pair, poly, prover_transcript);

    // initialize verifier transcript from proof data
    auto verifier_transcript = std::make_shared<NativeTranscript>(prover_transcript->proof_data);

    auto result = IPA::reduce_verify(this->vk(), opening_claim, verifier_transcript);
    EXPECT_TRUE(result);
}

namespace bb {
#if !defined(__wasm__)
// This test ensures that IPA throws or aborts when a challenge is zero, since it breaks the logic of the argument
TEST_F(IPATest, ChallengesAreZero)
{
    using IPA = IPA<Curve>;
    // generate a random polynomial, degree needs to be a power of two
    size_t n = 128;
    auto poly = this->random_polynomial(n);
    auto [x, eval] = this->random_eval(poly);
    auto commitment = this->commit(poly);
    const OpeningPair<Curve> opening_pair = { x, eval };
    const OpeningClaim<Curve> opening_claim{ opening_pair, commitment };

    // initialize an empty mock transcript
    auto transcript = std::make_shared<MockTranscript>();
    const size_t num_challenges = numeric::get_msb(n) + 1;
    std::vector<uint256_t> random_vector(num_challenges);

    // Generate a random element vector with challenges
    for (size_t i = 0; i < num_challenges; i++) {
        random_vector[i] = Fr::random_element();
    }

    // Compute opening proofs several times, where each time a different challenge is equal to zero. Should cause
    // exceptions
    for (size_t i = 0; i < num_challenges; i++) {
        auto new_random_vector = random_vector;
        new_random_vector[i] = Fr::zero();
        transcript->initialize(new_random_vector);
        EXPECT_ANY_THROW(IPA::compute_opening_proof_internal(this->ck(), opening_pair, poly, transcript));
    }
    // Fill out a vector of affine elements that the verifier receives from the prover with generators (we don't care
    // about them right now)
    std::vector<Curve::AffineElement> lrs(num_challenges * 2);
    for (size_t i = 0; i < num_challenges * 2; i++) {
        lrs[i] = Curve::AffineElement::one();
    }
    // Verify proofs several times, where each time a different challenge is equal to zero. Should cause
    // exceptions
    for (size_t i = 0; i < num_challenges; i++) {
        auto new_random_vector = random_vector;
        new_random_vector[i] = Fr::zero();
        transcript->initialize(new_random_vector, lrs, { uint256_t(n) });
        EXPECT_ANY_THROW(IPA::reduce_verify_internal(this->vk(), opening_claim, transcript));
    }
}

// This test checks that if the vector \vec{a_new} becomes zero after one round, it doesn't break IPA.
TEST_F(IPATest, AIsZeroAfterOneRound)
{
    using IPA = IPA<Curve>;
    // generate a random polynomial, degree needs to be a power of two
    size_t n = 4;
    auto poly = Polynomial(n);
    for (size_t i = 0; i < n / 2; i++) {
        poly[i] = Fr::random_element();
        poly[i + (n / 2)] = poly[i];
    }
    auto [x, eval] = this->random_eval(poly);
    auto commitment = this->commit(poly);
    const OpeningPair<Curve> opening_pair = { x, eval };
    const OpeningClaim<Curve> opening_claim{ opening_pair, commitment };

    // initialize an empty mock transcript
    auto transcript = std::make_shared<MockTranscript>();
    const size_t num_challenges = numeric::get_msb(n) + 1;
    std::vector<uint256_t> random_vector(num_challenges);

    // Generate a random element vector with challenges
    for (size_t i = 0; i < num_challenges; i++) {
        random_vector[i] = Fr::random_element();
    }
    // Substitute the first folding challenge with -1
    random_vector[1] = -Fr::one();

    // Put the challenges in the transcript
    transcript->initialize(random_vector);

    // Compute opening proof
    IPA::compute_opening_proof_internal(this->ck(), opening_pair, poly, transcript);

    // Reset indices
    transcript->reset_indices();

    // Verify
    EXPECT_TRUE(IPA::reduce_verify_internal(this->vk(), opening_claim, transcript));
}
#endif
} // namespace bb

TEST_F(IPATest, Commit)
{
    constexpr size_t n = 128;
    auto poly = this->random_polynomial(n);
    GroupElement commitment = this->commit(poly);
    auto* srs_elements = this->ck()->srs->get_monomial_points();
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
    auto prover_transcript = std::make_shared<NativeTranscript>();
    IPA::compute_opening_proof(this->ck(), opening_pair, poly, prover_transcript);

    // initialize verifier transcript from proof data
    auto verifier_transcript = std::make_shared<NativeTranscript>(prover_transcript->proof_data);

    auto result = IPA::reduce_verify(this->vk(), opening_claim, verifier_transcript);
    EXPECT_TRUE(result);

    EXPECT_EQ(prover_transcript->get_manifest(), verifier_transcript->get_manifest());
}

TEST_F(IPATest, GeminiShplonkIPAWithShift)
{
    using IPA = IPA<Curve>;
    using ShplonkProver = ShplonkProver_<Curve>;
    using ShplonkVerifier = ShplonkVerifier_<Curve>;
    using GeminiProver = GeminiProver_<Curve>;
    using GeminiVerifier = GeminiVerifier_<Curve>;

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

    auto prover_transcript = NativeTranscript::prover_init_empty();

    auto gemini_polynomials = GeminiProver::compute_gemini_polynomials(
        mle_opening_point, std::move(batched_unshifted), std::move(batched_to_be_shifted));

    for (size_t l = 0; l < log_n - 1; ++l) {
        std::string label = "FOLD_" + std::to_string(l + 1);
        auto commitment = this->ck()->commit(gemini_polynomials[l + 2]);
        prover_transcript->send_to_verifier(label, commitment);
    }

    const Fr r_challenge = prover_transcript->template get_challenge<Fr>("Gemini:r");

    const auto [gemini_opening_pairs, gemini_witnesses] = GeminiProver::compute_fold_polynomial_evaluations(
        mle_opening_point, std::move(gemini_polynomials), r_challenge);

    for (size_t l = 0; l < log_n; ++l) {
        std::string label = "Gemini:a_" + std::to_string(l);
        const auto& evaluation = gemini_opening_pairs[l + 1].evaluation;
        prover_transcript->send_to_verifier(label, evaluation);
    }

    const Fr nu_challenge = prover_transcript->template get_challenge<Fr>("Shplonk:nu");
    auto batched_quotient_Q =
        ShplonkProver::compute_batched_quotient(gemini_opening_pairs, gemini_witnesses, nu_challenge);
    prover_transcript->send_to_verifier("Shplonk:Q", this->ck()->commit(batched_quotient_Q));

    const Fr z_challenge = prover_transcript->template get_challenge<Fr>("Shplonk:z");
    const auto [shplonk_opening_pair, shplonk_witness] = ShplonkProver::compute_partially_evaluated_batched_quotient(
        gemini_opening_pairs, gemini_witnesses, std::move(batched_quotient_Q), nu_challenge, z_challenge);

    IPA::compute_opening_proof(this->ck(), shplonk_opening_pair, shplonk_witness, prover_transcript);

    auto verifier_transcript = NativeTranscript::verifier_init_empty(prover_transcript);

    auto gemini_verifier_claim = GeminiVerifier::reduce_verification(mle_opening_point,
                                                                     batched_evaluation,
                                                                     batched_commitment_unshifted,
                                                                     batched_commitment_to_be_shifted,
                                                                     verifier_transcript);

    const auto shplonk_verifier_claim =
        ShplonkVerifier::reduce_verification(this->vk(), gemini_verifier_claim, verifier_transcript);
    auto result = IPA::reduce_verify(this->vk(), shplonk_verifier_claim, verifier_transcript);

    EXPECT_EQ(result, true);
}
