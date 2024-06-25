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
namespace bb {
template <class Params> class ShplonkTest : public CommitmentTest<Params> {};

using CurveTypes = ::testing::Types<curve::BN254, curve::Grumpkin>;
TYPED_TEST_SUITE(ShplonkTest, CurveTypes);

// Test of Shplonk prover/verifier for two polynomials of different size, each opened at a single (different) point
TYPED_TEST(ShplonkTest, ShplonkSimple)
{
    using ShplonkProver = ShplonkProver_<TypeParam>;
    using ShplonkVerifier = ShplonkVerifier_<TypeParam>;
    using Fr = typename TypeParam::ScalarField;
    using ProverOpeningClaim = ProverOpeningClaim<TypeParam>;

    using OpeningClaim = OpeningClaim<TypeParam>;

    const size_t n = 16;

    auto prover_transcript = NativeTranscript::prover_init_empty();

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
    std::vector<ProverOpeningClaim> prover_opening_claims = { { poly1, { r1, eval1 } }, { poly2, { r2, eval2 } } };

    // Execute the shplonk prover functionality
    const auto batched_opening_claim = ShplonkProver::prove(this->ck(), prover_opening_claims, prover_transcript);
    // An intermediate check to confirm the opening of the shplonk prover witness Q
    this->verify_opening_pair(batched_opening_claim.opening_pair, batched_opening_claim.polynomial);

    // Aggregate polynomial commitments and their opening pairs
    std::vector<OpeningClaim> verifier_opening_claims = { { { r1, eval1 }, commitment1 },
                                                          { { r2, eval2 }, commitment2 } };

    auto verifier_transcript = NativeTranscript::verifier_init_empty(prover_transcript);

    // Execute the shplonk verifier functionality
    const auto batched_verifier_claim = ShplonkVerifier::reduce_verification(
        this->vk()->get_g1_identity(), verifier_opening_claims, verifier_transcript);

    this->verify_opening_claim(batched_verifier_claim, batched_opening_claim.polynomial);
}
} // namespace bb
