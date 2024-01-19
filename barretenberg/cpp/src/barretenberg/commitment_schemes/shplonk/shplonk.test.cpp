#include "shplonk.hpp"
#include "../gemini/gemini.hpp"

#include <algorithm>
#include <gtest/internal/gtest-internal.h>
#include <iterator>
#include <random>
#include <vector>

#include "../commitment_key.test.hpp"
#include "barretenberg/commitment_schemes/claim.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
namespace bb::honk::pcs::shplonk {
template <class Params> class ShplonkTest : public CommitmentTest<Params> {};

using CurveTypes = ::testing::Types<curve::BN254, curve::Grumpkin>;
TYPED_TEST_SUITE(ShplonkTest, CurveTypes);

// Test of Shplonk prover/verifier for two polynomials of different size, each opened at a single (different) point
TYPED_TEST(ShplonkTest, ShplonkSimple)
{
    using ShplonkProver = ShplonkProver_<TypeParam>;
    using ShplonkVerifier = ShplonkVerifier_<TypeParam>;
    using Fr = typename TypeParam::ScalarField;
    using Polynomial = typename bb::Polynomial<Fr>;
    using OpeningPair = OpeningPair<TypeParam>;
    using OpeningClaim = OpeningClaim<TypeParam>;

    const size_t n = 16;

    auto prover_transcript = BaseTranscript::prover_init_empty();

    // Generate two random (unrelated) polynomials of two different sizes, as well as their evaluations at a (single but
    // different) random point and their commitments.
    const auto r1 = Fr::random_element();
    auto poly1 = this->random_polynomial(n);
    const auto eval1 = poly1.evaluate(r1);
    const auto commitment1 = this->commit(poly1);

    const auto r2 = Fr::random_element();
    auto poly2 = this->random_polynomial(n / 2);
    const auto eval2 = poly2.evaluate(r2);
    const auto commitment2 = this->commit(poly2);

    // Aggregate polynomials and their opening pairs
    std::vector<OpeningPair> opening_pairs = { { r1, eval1 }, { r2, eval2 } };
    std::vector<Polynomial> polynomials = { poly1.share(), poly2.share() };

    // Execute the shplonk prover functionality
    const Fr nu_challenge = prover_transcript->get_challenge("Shplonk:nu");
    auto batched_quotient_Q = ShplonkProver::compute_batched_quotient(opening_pairs, polynomials, nu_challenge);
    prover_transcript->send_to_verifier("Shplonk:Q", this->ck()->commit(batched_quotient_Q));

    const Fr z_challenge = prover_transcript->get_challenge("Shplonk:z");
    const auto [prover_opening_pair, shplonk_prover_witness] =
        ShplonkProver::compute_partially_evaluated_batched_quotient(
            opening_pairs, polynomials, std::move(batched_quotient_Q), nu_challenge, z_challenge);

    // An intermediate check to confirm the opening of the shplonk prover witness Q
    this->verify_opening_pair(prover_opening_pair, shplonk_prover_witness);

    // Aggregate polynomial commitments and their opening pairs
    std::vector<OpeningClaim> opening_claims;
    opening_claims.emplace_back(OpeningClaim{ opening_pairs[0], commitment1 });
    opening_claims.emplace_back(OpeningClaim{ opening_pairs[1], commitment2 });

    auto verifier_transcript = BaseTranscript::verifier_init_empty(prover_transcript);

    // Execute the shplonk verifier functionality
    const auto verifier_claim = ShplonkVerifier::reduce_verification(this->vk(), opening_claims, verifier_transcript);

    this->verify_opening_claim(verifier_claim, shplonk_prover_witness);
}
} // namespace bb::honk::pcs::shplonk
