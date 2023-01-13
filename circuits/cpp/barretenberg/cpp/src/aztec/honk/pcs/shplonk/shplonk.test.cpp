#include "shplonk_multi.hpp"
#include "shplonk_single.hpp"
#include "../gemini/gemini.hpp"

#include <random>
#include <iterator>
#include <algorithm>

#include "../commitment_key.test.hpp"
#include "honk/pcs/claim.hpp"
#include "polynomials/polynomial.hpp"
namespace honk::pcs::shplonk {
template <class Params> class ShplonkTest : public CommitmentTest<Params> {
    using Base = CommitmentTest<Params>;

    using Fr = typename Params::Fr;

    using Commitment = typename Params::Commitment;
    using Polynomial = barretenberg::Polynomial<Fr>;

  public:
    std::vector<Fr> random_opening_set(const size_t m)
    {
        std::vector<Fr> opening_set(m);
        for (size_t i = 0; i < m; ++i) {
            auto x = this->random_element();
            opening_set[i] = x;
        }
        return opening_set;
    }

    void add_random_batch_opening_sub_claims(std::vector<MultiOpeningClaim<Params>>& multi_claims,
                                             std::vector<Polynomial>& polys,
                                             const std::vector<Fr>& queries,
                                             std::span<const size_t> poly_sizes)
    {
        using Opening = typename MultiOpeningClaim<Params>::Opening;
        auto& new_claim = multi_claims.emplace_back(MultiOpeningClaim<Params>{ queries, {} });

        for (const size_t poly_size : poly_sizes) {
            const auto& p = polys.emplace_back(Base::random_polynomial(poly_size));
            const auto& c = Base::commit(p);
            std::vector<Fr> evals;
            for (const auto query : queries) {
                evals.push_back(p.evaluate(query));
            }
            new_claim.openings.emplace_back(Opening{ c, evals });
        }
    };
};

TYPED_TEST_SUITE(ShplonkTest, CommitmentSchemeParams);

TYPED_TEST(ShplonkTest, single_poly_two_points)
{
    using Shplonk = MultiBatchOpeningScheme<TypeParam>;
    using MultiOpeningClaim = MultiOpeningClaim<TypeParam>;
    using Fr = typename TypeParam::Fr;
    using Polynomial = barretenberg::Polynomial<Fr>;
    constexpr size_t n = 16;
    const size_t log_n = 4;

    auto queries = this->random_opening_set(2);
    std::vector<MultiOpeningClaim> claims;
    std::vector<Polynomial> polys;

    this->add_random_batch_opening_sub_claims(claims, polys, queries, std::array{ n });

    using Transcript = transcript::StandardTranscript;
    auto transcript = std::make_shared<Transcript>(StandardHonk::create_unrolled_manifest(0, log_n));

    transcript->mock_inputs_prior_to_challenge("nu");

    const auto [prover_claim, witness, proof] = Shplonk::reduce_prove(this->ck(), claims, polys, transcript);

    this->verify_opening_claim(prover_claim, witness);
    const auto verifier_claim = Shplonk::reduce_verify(claims, proof, transcript);
    EXPECT_EQ(prover_claim, verifier_claim);
    this->verify_opening_claim(prover_claim, witness);
}

TYPED_TEST(ShplonkTest, two_polys_different_size_at_two_different_points)
{
    using Shplonk = MultiBatchOpeningScheme<TypeParam>;
    using MultiOpeningClaim = MultiOpeningClaim<TypeParam>;
    using Fr = typename TypeParam::Fr;
    using Polynomial = barretenberg::Polynomial<Fr>;
    const size_t n = 16;
    const size_t log_n = 4;

    std::vector<MultiOpeningClaim> claims;
    std::vector<Polynomial> polys;

    auto queries = this->random_opening_set(2);

    this->add_random_batch_opening_sub_claims(claims, polys, { queries[0] }, std::array{ n });
    this->add_random_batch_opening_sub_claims(claims, polys, { queries[1] }, std::array{ n - 1 });

    using Transcript = transcript::StandardTranscript;
    auto transcript = std::make_shared<Transcript>(StandardHonk::create_unrolled_manifest(0, log_n));

    transcript->mock_inputs_prior_to_challenge("nu");

    const auto [prover_claim, witness, proof] = Shplonk::reduce_prove(this->ck(), claims, polys, transcript);

    this->verify_opening_claim(prover_claim, witness);
    const auto verifier_claim = Shplonk::reduce_verify(claims, proof, transcript);
    EXPECT_EQ(prover_claim, verifier_claim);
    this->verify_opening_claim(prover_claim, witness);
}

TYPED_TEST(ShplonkTest, three_polys_different_sizes_and_different_queries)
{
    using Shplonk = MultiBatchOpeningScheme<TypeParam>;
    using MultiOpeningClaim = MultiOpeningClaim<TypeParam>;
    using Fr = typename TypeParam::Fr;
    using Polynomial = barretenberg::Polynomial<Fr>;
    const size_t n = 16;
    const size_t log_n = 4;

    std::vector<MultiOpeningClaim> claims;
    std::vector<Polynomial> polys;

    auto queries = this->random_opening_set(3);

    this->add_random_batch_opening_sub_claims(claims, polys, { queries[0] }, std::array{ n });
    this->add_random_batch_opening_sub_claims(claims, polys, { queries[1], queries[2] }, std::array{ n - 1, n + 2 });
    this->add_random_batch_opening_sub_claims(claims, polys, { queries[0], queries[2] }, std::array{ n });

    using Transcript = transcript::StandardTranscript;
    auto transcript = std::make_shared<Transcript>(StandardHonk::create_unrolled_manifest(0, log_n));

    transcript->mock_inputs_prior_to_challenge("nu");

    const auto [prover_claim, witness, proof] = Shplonk::reduce_prove(this->ck(), claims, polys, transcript);

    this->verify_opening_claim(prover_claim, witness);
    const auto verifier_claim = Shplonk::reduce_verify(claims, proof, transcript);
    EXPECT_EQ(prover_claim, verifier_claim);
    this->verify_opening_claim(prover_claim, witness);
}

// Test of Shplonk prover/verifier using real Gemini claim
TYPED_TEST(ShplonkTest, Gemini)
{
    using Shplonk = SingleBatchOpeningScheme<TypeParam>;
    using Gemini = gemini::MultilinearReductionScheme<TypeParam>;
    using MLEOpeningClaim = MLEOpeningClaim<TypeParam>;

    const size_t n = 16;
    const size_t log_n = 4;

    const auto u = this->random_evaluation_point(log_n);
    auto poly = this->random_polynomial(n);
    const auto commitment = this->commit(poly);
    const auto eval = poly.evaluate_mle(u);

    // create opening claim
    const auto claims = { MLEOpeningClaim{ commitment, eval } };

    using Transcript = transcript::StandardTranscript;
    auto transcript = std::make_shared<Transcript>(StandardHonk::create_unrolled_manifest(0, log_n));

    transcript->mock_inputs_prior_to_challenge("rho");

    auto [gemini_claim, gemini_witness, gemini_proof] =
        Gemini::reduce_prove(this->ck(), u, claims, {}, { &poly }, {}, transcript);

    Gemini::reduce_verify(u, claims, {}, gemini_proof, transcript);

    const auto [prover_claim, witness, proof] =
        Shplonk::reduce_prove(this->ck(), gemini_claim, gemini_witness, transcript);

    this->verify_opening_claim(prover_claim, witness);

    const auto verifier_claim = Shplonk::reduce_verify(gemini_claim, proof, transcript);
    EXPECT_EQ(prover_claim, verifier_claim);
}
} // namespace honk::pcs::shplonk