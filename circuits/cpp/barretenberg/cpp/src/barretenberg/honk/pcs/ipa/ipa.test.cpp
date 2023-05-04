#include "ipa.hpp"
#include "barretenberg/common/mem.hpp"
#include <gtest/gtest.h>
#include "barretenberg/polynomials/polynomial_arithmetic.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/ecc/curves/bn254/fq12.hpp"
#include "barretenberg/honk/pcs/commitment_key.hpp"
#include "barretenberg/honk/pcs/commitment_key.test.hpp"
using namespace barretenberg;
namespace proof_system::honk::pcs {

class IPATests : public CommitmentTest<ipa::Params> {
  public:
    using Params = ipa::Params;
    using Fr = typename Params::Fr;
    using element = typename Params::Commitment;
    using affine_element = typename Params::C;
    using CK = typename Params::CK;
    using VK = typename Params::VK;
    using Polynomial = barretenberg::Polynomial<Fr>;
};

TEST_F(IPATests, Commit)
{
    constexpr size_t n = 128;
    auto poly = this->random_polynomial(n);
    barretenberg::g1::element commitment = this->commit(poly);
    auto srs_elements = this->ck()->srs.get_monomial_points();
    barretenberg::g1::element expected = srs_elements[0] * poly[0];
    for (size_t i = 1; i < n; i++) {
        expected += srs_elements[i] * poly[i];
    }
    EXPECT_EQ(expected.normalize(), commitment.normalize());
}

TEST_F(IPATests, Open)
{
    using IPA = ipa::InnerProductArgument<Params>;
    // generate a random polynomial, degree needs to be a power of two
    size_t n = 128;
    auto poly = this->random_polynomial(n);
    auto [x, eval] = this->random_eval(poly);
    auto commitment = this->commit(poly);
    const OpeningPair<Params> opening_pair{ x, eval };

    // initialize empty prover transcript
    ProverTranscript<Fr> prover_transcript;

    prover_transcript.send_to_verifier("IPA:C", commitment);

    IPA::reduce_prove(this->ck(), opening_pair, poly, prover_transcript);

    // initialize verifier transcript from proof data
    VerifierTranscript<Fr> verifier_transcript{ prover_transcript.proof_data };

    auto result = IPA::reduce_verify(this->vk(), opening_pair, n, verifier_transcript);
    EXPECT_TRUE(result);

    EXPECT_EQ(prover_transcript.get_manifest(), verifier_transcript.get_manifest());
}
} // namespace proof_system::honk::pcs
