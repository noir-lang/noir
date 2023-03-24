#include "shplonk_single.hpp"
#include "../gemini/gemini.hpp"

#include <gtest/internal/gtest-internal.h>
#include <random>
#include <iterator>
#include <algorithm>

#include "../commitment_key.test.hpp"
#include "barretenberg/honk/pcs/claim.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
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

    auto prover_transcript = ProverTranscript<Fr>::init_empty();

    const Fr rho = Fr::random_element();

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

    auto gemini_prover_output = Gemini::reduce_prove(
        this->ck(), u, std::move(batched_unshifted), std::move(batched_to_be_shifted), prover_transcript);

    const auto [prover_opening_pair, shplonk_prover_witness] = Shplonk::reduce_prove(
        this->ck(), gemini_prover_output.opening_pairs, gemini_prover_output.witnesses, prover_transcript);

    this->verify_opening_pair(prover_opening_pair, shplonk_prover_witness);

    auto verifier_transcript = VerifierTranscript<Fr>::init_empty(prover_transcript);

    auto gemini_verifier_claim = Gemini::reduce_verify(
        u, batched_evaluation, batched_commitment_unshifted, batched_commitment_to_be_shifted, verifier_transcript);

    const auto verifier_claim = Shplonk::reduce_verify(gemini_verifier_claim, verifier_transcript);

    this->verify_opening_claim(verifier_claim, shplonk_prover_witness);
}
} // namespace honk::pcs::shplonk
