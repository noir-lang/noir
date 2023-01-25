#include "gemini.hpp"

#include "../commitment_key.test.hpp"
#include <gtest/gtest.h>

namespace honk::pcs::gemini {

template <class Params> class GeminiTest : public CommitmentTest<Params> {};

TYPED_TEST_SUITE(GeminiTest, CommitmentSchemeParams);

TYPED_TEST(GeminiTest, single)
{
    using Gemini = MultilinearReductionScheme<TypeParam>;
    using MLEOpeningClaim = MLEOpeningClaim<TypeParam>;

    const size_t n = 16;
    const size_t log_n = 4;

    auto u = this->random_evaluation_point(log_n);
    auto poly = this->random_polynomial(n);
    auto commitment = this->commit(poly);
    auto eval = poly.evaluate_mle(u);

    // create opening claim
    auto claims = { MLEOpeningClaim{ commitment, eval } };

    using Transcript = transcript::StandardTranscript;
    auto transcript = std::make_shared<Transcript>(StandardHonk::create_unrolled_manifest(0, log_n));

    transcript->mock_inputs_prior_to_challenge("rho");

    auto [prover_claim, witness, proof] = Gemini::reduce_prove(this->ck(), u, claims, {}, { &poly }, {}, transcript);

    this->verify_batch_opening_claim(prover_claim, witness);

    auto verifier_claim = Gemini::reduce_verify(u, claims, {}, proof, transcript);

    this->verify_batch_opening_claim(verifier_claim, witness);

    EXPECT_EQ(prover_claim, verifier_claim);
}

TYPED_TEST(GeminiTest, shift)
{
    using Gemini = MultilinearReductionScheme<TypeParam>;
    using Fr = typename TypeParam::Fr;
    using MLEOpeningClaim = MLEOpeningClaim<TypeParam>;

    const size_t n = 16;
    const size_t log_n = 4;

    auto u = this->random_evaluation_point(log_n);

    // shiftable polynomial must have 0 as last coefficient
    auto poly = this->random_polynomial(n);
    poly[0] = Fr::zero();

    auto commitment = this->commit(poly);
    auto eval_shift = poly.evaluate_mle(u, true);

    // create opening claim
    auto claims_shift = {
        MLEOpeningClaim{ commitment, eval_shift },
    };

    using Transcript = transcript::StandardTranscript;
    auto transcript = std::make_shared<Transcript>(StandardHonk::create_unrolled_manifest(0, log_n));

    transcript->mock_inputs_prior_to_challenge("rho");

    auto [prover_claim, witness, proof] =
        Gemini::reduce_prove(this->ck(), u, {}, claims_shift, {}, { &poly }, transcript);

    this->verify_batch_opening_claim(prover_claim, witness);

    auto verifier_claim = Gemini::reduce_verify(u, {}, claims_shift, proof, transcript);

    EXPECT_EQ(prover_claim, verifier_claim);
}

TYPED_TEST(GeminiTest, double)
{
    using Gemini = MultilinearReductionScheme<TypeParam>;
    using MLEOpeningClaim = MLEOpeningClaim<TypeParam>;

    const size_t n = 16;
    const size_t log_n = 4;

    auto u = this->random_evaluation_point(log_n);

    auto poly1 = this->random_polynomial(n);
    auto poly2 = this->random_polynomial(n);

    auto commitment1 = this->commit(poly1);
    auto commitment2 = this->commit(poly2);

    auto eval1 = poly1.evaluate_mle(u);
    auto eval2 = poly2.evaluate_mle(u);

    const auto claims = {
        MLEOpeningClaim{ commitment1, eval1 },
        MLEOpeningClaim{ commitment2, eval2 },
    };

    using Transcript = transcript::StandardTranscript;
    auto transcript = std::make_shared<Transcript>(StandardHonk::create_unrolled_manifest(0, log_n));

    transcript->mock_inputs_prior_to_challenge("rho");

    auto [prover_claim, witness, proof] =
        Gemini::reduce_prove(this->ck(), u, claims, {}, { &poly1, &poly2 }, {}, transcript);

    this->verify_batch_opening_claim(prover_claim, witness);

    auto verifier_claim = Gemini::reduce_verify(u, claims, {}, proof, transcript);

    this->verify_batch_opening_claim(verifier_claim, witness);
    EXPECT_EQ(prover_claim, verifier_claim);
}

TYPED_TEST(GeminiTest, double_shift)
{
    using Gemini = MultilinearReductionScheme<TypeParam>;
    using Fr = typename TypeParam::Fr;
    using MLEOpeningClaim = MLEOpeningClaim<TypeParam>;

    const size_t n = 16;
    const size_t log_n = 4;

    auto u = this->random_evaluation_point(log_n);

    auto poly1 = this->random_polynomial(n);
    auto poly2 = this->random_polynomial(n);
    poly2[0] = Fr::zero();

    auto commitment1 = this->commit(poly1);
    auto commitment2 = this->commit(poly2);

    auto eval1 = poly1.evaluate_mle(u);
    auto eval2 = poly2.evaluate_mle(u);
    auto eval2_shift = poly2.evaluate_mle(u, true);

    auto claims = {
        MLEOpeningClaim{ commitment1, eval1 },
        MLEOpeningClaim{ commitment2, eval2 },
    };

    auto claims_shift = {
        MLEOpeningClaim{ commitment2, eval2_shift },
    };

    using Transcript = transcript::StandardTranscript;
    auto transcript = std::make_shared<Transcript>(StandardHonk::create_unrolled_manifest(0, log_n));

    transcript->mock_inputs_prior_to_challenge("rho");

    auto [prover_claim, witness, proof] =
        Gemini::reduce_prove(this->ck(), u, claims, claims_shift, { &poly1, &poly2 }, { &poly2 }, transcript);

    this->verify_batch_opening_claim(prover_claim, witness);

    auto verifier_claim = Gemini::reduce_verify(u, claims, claims_shift, proof, transcript);

    ASSERT_EQ(prover_claim, verifier_claim);
}

} // namespace honk::pcs::gemini