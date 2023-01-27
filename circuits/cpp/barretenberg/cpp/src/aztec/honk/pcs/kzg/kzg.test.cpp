
#include "kzg.hpp"
#include "../shplonk/shplonk_single.hpp"
#include "../gemini/gemini.hpp"

#include "../commitment_key.test.hpp"
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

    using Accumulator = BilinearAccumulator<Params>;

    void verify_accumulators(const Accumulator& prover_acc, const Accumulator& verifier_acc)
    {
        EXPECT_EQ(prover_acc, verifier_acc) << "BilinearAccumulation: accumulator mismatch";
        EXPECT_TRUE(prover_acc.verify(this->vk())) << "BilinearAccumulation: pairing check failed";
    }
};

TYPED_TEST_SUITE(BilinearAccumulationTest, CommitmentSchemeParams);

TYPED_TEST(BilinearAccumulationTest, single)
{
    const size_t n = 16;

    using OpeningScheme = UnivariateOpeningScheme<TypeParam>;

    auto [claim, witness] = this->random_claim(n);

    auto [acc, proof] = OpeningScheme::reduce_prove(this->ck(), claim, witness);
    auto result_acc = OpeningScheme::reduce_verify(claim, proof);

    this->verify_accumulators(acc, result_acc);
}

/**
 * @brief Test full PCS protocol: Gemini, Shplonk, KZG and pairing check
 * @details This test serves two purposes:
 * (1) Demonstrate the full PCS protocol as it is used in the construction and verification
 * of a single Honk proof. (Expository comments included throughout).
 * (2) Demonstrate that proof construction/verification does not require the prover to pass
 * genuine claims to the PCS. (This is relevant since in practice the prover does not have
 * access to, for example, commitments to non-witness polynomials). The prover must provide
 * only the multivariate polynomials and their genuine evaluations to the PCS, not commitments
 * to those polynomials.
 *
 */
TYPED_TEST(BilinearAccumulationTest, GeminiShplonkKzgWithShift)
{
    using Transcript = transcript::StandardTranscript;
    using Shplonk = shplonk::SingleBatchOpeningScheme<TypeParam>;
    using Gemini = gemini::MultilinearReductionScheme<TypeParam>;
    using MLEOpeningClaim = MLEOpeningClaim<TypeParam>;
    using OpeningScheme = UnivariateOpeningScheme<TypeParam>;

    const size_t n = 16;
    const size_t log_n = 4;

    // Instantiate a transcript from the real Honk manifest, then mock the inputs prior to Gemini.
    auto transcript = std::make_shared<Transcript>(StandardHonk::create_unrolled_manifest(0, log_n));
    transcript->mock_inputs_prior_to_challenge("rho");

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

    const auto mock_commitment = Params::C::one();

    std::vector<MLEOpeningClaim> claims;
    std::vector<MLEOpeningClaim> claims_shift;
    std::vector<MLEOpeningClaim> claims_mock;
    std::vector<MLEOpeningClaim> claims_shift_mock;
    std::vector<Params::Polynomial*> multivariate_polynomials;
    std::vector<Params::Polynomial*> multivariate_polynomials_shifted;

    // Create genuine opening claims (for use by verifier) and mock opening claims (for prover)
    claims.emplace_back(commitment1, eval1);
    claims.emplace_back(commitment2, eval2);
    claims_shift.emplace_back(commitment2, eval2_shift);

    claims_mock.emplace_back(mock_commitment, eval1);
    claims_mock.emplace_back(mock_commitment, eval2);
    claims_shift_mock.emplace_back(mock_commitment, eval2_shift);

    multivariate_polynomials.emplace_back(&poly1);
    multivariate_polynomials.emplace_back(&poly2);
    multivariate_polynomials_shifted.emplace_back(&poly2);

    // Run the full prover PCS protocol with mocked opening claims (mocked commitment, genuine evaluation)

    // Gemini prover output:
    // - claim: junk commitments, d+1 genuine evaluations a_0_pos, a_l, l = 0:d-1
    // - witness: the d+1 polynomials Fold_{r}^(0), Fold_{-r}^(0), Fold^(l), l = 1:d-1
    // - proof: d-1 commitments [Fold^(l)], l = 1:d-1 and d evaluations a_l, l = 0:d-1
    const auto [gemini_prover_claim, gemini_witness, gemini_proof] =
        Gemini::reduce_prove(this->ck(),
                             mle_opening_point,
                             claims_mock,
                             claims_shift_mock,
                             multivariate_polynomials,
                             multivariate_polynomials_shifted,
                             transcript);

    // Shplonk prover output:
    // - claim: junk commitment, evaluation point = zero
    // - witness: polynomial Q - Q_z
    // - proof: commitment [Q]
    const auto [shplonk_prover_claim, shplonk_witness, shplonk_proof] =
        Shplonk::reduce_prove(this->ck(), gemini_prover_claim, gemini_witness, transcript);

    // KZG prover output:
    // - proof: commitment [W]
    auto [kzg_accum, kzg_proof] = OpeningScheme::reduce_prove(this->ck(), shplonk_prover_claim, shplonk_witness);

    // Run the full verifier PCS protocol with genuine opening claims (genuine commitment, genuine evaluation)

    // Gemini verifier output:
    // - claim: d+1 commitments to Fold_{r}^(0), Fold_{-r}^(0), Fold^(l), d+1 evaluations a_0_pos, a_l, l = 0:d-1
    const auto gemini_verifier_claim =
        Gemini::reduce_verify(mle_opening_point, claims, claims_shift, gemini_proof, transcript);

    // Shplonk verifier output:
    // - claim: commitment [Q] - [Q_z], evaluation zero (at random challenge z)
    const auto shplonk_verifier_claim = Shplonk::reduce_verify(gemini_verifier_claim, shplonk_proof, transcript);

    // KZG verifier output:
    // - just aggregates inputs [Q] - [Q_z] and [W] into an 'accumulator' (can perform pairing check on result)
    auto kzg_claim = OpeningScheme::reduce_verify(shplonk_verifier_claim, kzg_proof);

    // Final pairing check: e([Q] - [Q_z] + z[W], [1]_2) = e([W], [x]_2)
    bool verified = kzg_claim.verify(this->vk());

    EXPECT_EQ(verified, true);
}

TYPED_TEST(BilinearAccumulationTest, GeminiShplonkKzgSimple)
{
    using Transcript = transcript::StandardTranscript;
    using Shplonk = shplonk::SingleBatchOpeningScheme<TypeParam>;
    using Gemini = gemini::MultilinearReductionScheme<TypeParam>;
    using MLEOpeningClaim = MLEOpeningClaim<TypeParam>;
    using OpeningScheme = UnivariateOpeningScheme<TypeParam>;

    const size_t n = 16;
    const size_t log_n = 4;

    // Instantiate a transcript from the real Honk manifest, then mock the inputs prior to Gemini.
    auto transcript = std::make_shared<Transcript>(StandardHonk::create_unrolled_manifest(0, log_n));
    transcript->mock_inputs_prior_to_challenge("rho");

    // Generate multilinear polynomials, their commitments (genuine and mocked) and evaluations (genuine) at a random
    // point.
    const auto mle_opening_point = this->random_evaluation_point(log_n); // sometimes denoted 'u'
    auto poly1 = this->random_polynomial(n);

    auto commitment1 = this->commit(poly1);

    auto eval1 = poly1.evaluate_mle(mle_opening_point);

    const auto mock_commitment = Params::C::one();

    std::vector<MLEOpeningClaim> claims;
    std::vector<MLEOpeningClaim> claims_mock;
    std::vector<Params::Polynomial*> multivariate_polynomials;

    claims.emplace_back(commitment1, eval1);
    claims_mock.emplace_back(mock_commitment, eval1);
    multivariate_polynomials.emplace_back(&poly1);

    // Run the full prover PCS protocol with mocked opening claims (mocked commitment, genuine evaluation)

    // Gemini prover output:
    // - claim: junk commitments, d+1 genuine evaluations a_0_pos, a_l, l = 0:d-1
    // - witness: the d+1 polynomials Fold_{r}^(0), Fold_{-r}^(0), Fold^(l), l = 1:d-1
    // - proof: d-1 commitments [Fold^(l)], l = 1:d-1 and d evaluations a_l, l = 0:d-1
    const auto [gemini_prover_claim, gemini_witness, gemini_proof] =
        Gemini::reduce_prove(this->ck(), mle_opening_point, claims_mock, {}, multivariate_polynomials, {}, transcript);

    // Shplonk prover output:
    // - claim: junk commitment, evaluation point = zero
    // - witness: polynomial Q - Q_z
    // - proof: commitment [Q]
    const auto [shplonk_prover_claim, shplonk_witness, shplonk_proof] =
        Shplonk::reduce_prove(this->ck(), gemini_prover_claim, gemini_witness, transcript);

    // KZG prover output:
    // - proof: commitment [W]
    auto [kzg_accum, kzg_proof] = OpeningScheme::reduce_prove(this->ck(), shplonk_prover_claim, shplonk_witness);

    // Run the full verifier PCS protocol with genuine opening claims (genuine commitment, genuine evaluation)

    // Gemini verifier output:
    // - claim: d+1 commitments to Fold_{r}^(0), Fold_{-r}^(0), Fold^(l), d+1 evaluations a_0_pos, a_l, l = 0:d-1
    const auto gemini_verifier_claim = Gemini::reduce_verify(mle_opening_point, claims, {}, gemini_proof, transcript);

    // Shplonk verifier output:
    // - claim: commitment [Q] - [Q_z], evaluation zero (at random challenge z)
    const auto shplonk_verifier_claim = Shplonk::reduce_verify(gemini_verifier_claim, shplonk_proof, transcript);

    // KZG verifier output:
    // - just aggregates inputs [Q] - [Q_z] and [W] into an 'accumulator' (can perform pairing check on result)
    auto kzg_claim = OpeningScheme::reduce_verify(shplonk_verifier_claim, kzg_proof);

    // Final pairing check: e([Q] - [Q_z] + z[W], [1]_2) = e([W], [x]_2)
    bool verified = kzg_claim.verify(this->vk());

    EXPECT_EQ(verified, true);
}

} // namespace honk::pcs::kzg