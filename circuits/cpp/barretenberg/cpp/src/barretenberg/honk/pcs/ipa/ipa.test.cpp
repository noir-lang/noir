#include "ipa.hpp"
#include "barretenberg/common/mem.hpp"
#include <gtest/gtest.h>
#include "barretenberg/polynomials/polynomial_arithmetic.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/ecc/curves/bn254/fq12.hpp"
#include "barretenberg/honk/pcs/commitment_key.hpp"
#include "barretenberg/honk/pcs/commitment_key.test.hpp"
using namespace barretenberg;
namespace proof_system::honk::pcs::ipa {

template <class Params> class IpaCommitmentTest : public CommitmentTest<Params> {
    using Fr = typename Params::Fr;
    using element = typename Params::Commitment;
    using affine_element = typename Params::C;
    using CK = typename Params::CK;
    using VK = typename Params::VK;
    using Polynomial = barretenberg::Polynomial<Fr>;
};

TYPED_TEST_SUITE(IpaCommitmentTest, IpaCommitmentSchemeParams);

TYPED_TEST(IpaCommitmentTest, commit)
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

TYPED_TEST(IpaCommitmentTest, open)
{
    using IPA = InnerProductArgument<TypeParam>;
    using PubInput = typename IPA::PubInput;
    // generate a random polynomial, degree needs to be a power of two
    size_t n = 128;
    auto poly = this->random_polynomial(n);
    auto [x, eval] = this->random_eval(poly);
    barretenberg::g1::element commitment = this->commit(poly);
    PubInput pub_input;
    pub_input.commitment = commitment;
    pub_input.challenge_point = x;
    pub_input.evaluation = eval;
    pub_input.poly_degree = n;
    auto aux_scalar = fr::random_element();
    pub_input.aux_generator = barretenberg::g1::one * aux_scalar;
    const size_t log_n = static_cast<size_t>(numeric::get_msb(n));
    pub_input.round_challenges = std::vector<barretenberg::fr>(log_n);
    for (size_t i = 0; i < log_n; i++) {
        pub_input.round_challenges[i] = barretenberg::fr::random_element();
    }
    auto proof = IPA::reduce_prove(this->ck(), pub_input, poly);
    auto result = IPA::reduce_verify(this->vk(), proof, pub_input);
    EXPECT_TRUE(result);
}
} // namespace proof_system::honk::pcs::ipa
