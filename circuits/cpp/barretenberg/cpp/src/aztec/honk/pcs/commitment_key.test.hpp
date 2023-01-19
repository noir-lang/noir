#pragma once

#include <cstddef>
#include <gtest/gtest.h>

#include <concepts>
#include <algorithm>
#include <memory>
#include <string_view>

#include <polynomials/polynomial.hpp>
#include <srs/reference_string/file_reference_string.hpp>
#include <ecc/curves/bn254/g1.hpp>

#include "../oracle/oracle.hpp"
#include "../../transcript/transcript_wrappers.hpp"
#include "../../proof_system/flavor/flavor.hpp"

#include "claim.hpp"
#include "commitment_key.hpp"

namespace honk::pcs {
namespace {
constexpr std::string_view kzg_srs_path = "../srs_db/ignition";
}

template <class CK> inline std::shared_ptr<CK> CreateCommitmentKey();

template <> inline std::shared_ptr<kzg::CommitmentKey> CreateCommitmentKey<kzg::CommitmentKey>()
{
    const size_t n = 128;
    return std::make_shared<kzg::CommitmentKey>(n, kzg_srs_path);
}

template <typename CK> inline std::shared_ptr<CK> CreateCommitmentKey()
// requires std::default_initializable<CK>
{
    return std::make_shared<CK>();
}

template <class VK> inline VK* CreateVerificationKey();

template <> inline kzg::VerificationKey* CreateVerificationKey<kzg::VerificationKey>()
{
    return new kzg::VerificationKey(kzg_srs_path);
}

template <typename VK> inline VK* CreateVerificationKey()
// requires std::default_initializable<VK>
{
    return new VK();
}

template <typename Params> class CommitmentTest : public ::testing::Test {
    using CK = typename Params::CK;
    using VK = typename Params::VK;

    using Fr = typename Params::Fr;
    using Commitment = typename Params::Commitment;
    using Polynomial = typename Params::Polynomial;
    using Transcript = transcript::StandardTranscript;

  public:
    CommitmentTest()
        : engine{ &numeric::random::get_debug_engine() }
    {}

    std::shared_ptr<CK> ck() { return commitment_key; }
    VK* vk() { return verification_key; }

    Commitment commit(const Polynomial& polynomial) { return commitment_key->commit(polynomial); }

    Polynomial random_polynomial(const size_t n)
    {
        Polynomial p(n, n);
        for (size_t i = 0; i < n; ++i) {
            p[i] = Fr::random_element(engine);
        }
        return p;
    }

    Fr random_element() { return Fr::random_element(engine); }

    std::pair<Fr, Fr> random_eval(const Polynomial& polynomial)
    {
        Fr x{ random_element() };
        Fr y{ polynomial.evaluate(x) };
        return { x, y };
    }

    std::pair<OpeningClaim<Params>, Polynomial> random_claim(const size_t n)
    {
        auto p = random_polynomial(n);
        auto [x, y] = random_eval(p);
        auto c = commit(p);
        return { { c, x, y }, p };
    };

    std::vector<Fr> random_evaluation_point(const size_t num_variables)
    {
        std::vector<Fr> u(num_variables);
        for (size_t l = 0; l < num_variables; ++l) {
            u[l] = random_element();
        }
        return u;
    }

    void verify_opening_claim(const OpeningClaim<Params>& claim, const Polynomial& witness)
    {
        auto& [c, x, y] = claim;
        Fr y_expected = witness.evaluate(x);
        EXPECT_EQ(y, y_expected) << "OpeningClaim: evaluations mismatch";
        Commitment c_expected = commit(witness);
        EXPECT_EQ(c, c_expected) << "OpeningClaim: commitment mismatch";
    }

    /**
     * @brief Ensures that a 'BatchOpeningClaim' is correct by checking that
     * - all evaluations are correct by recomputing them from each witness polynomial.
     * - commitments are correct by recomputing a commitment from each witness polynomial.
     * - each 'queries' is a subset of 'all_queries' and 'all_queries' is the union of all 'queries'
     * - each 'commitment' of each 'SubClaim' appears only once.
     */
    void verify_batch_opening_claim(std::span<const MultiOpeningClaim<Params>> multi_claims,
                                    std::span<const Polynomial> witnesses)
    {
        size_t idx = 0;

        for (const auto& [queries, openings] : multi_claims) {
            const size_t num_queries = queries.size();

            for (const auto& [commitment, evals] : openings) {
                // compare commitment against recomputed commitment from witness
                Commitment commitment_expected = commit(witnesses[idx]);
                EXPECT_EQ(commitment, commitment_expected)
                    << "BatchOpeningClaim idx=" << idx << ": commitment mismatch";
                EXPECT_EQ(evals.size(), num_queries)
                    << "BatchOpeningClaim idx=" << idx << ": evaluation/query size mismatch";

                // check evaluations for each point in queries
                for (size_t i = 0; i < num_queries; ++i) {

                    // check evaluation
                    Fr eval_expected = witnesses[idx].evaluate(queries[i]);
                    EXPECT_EQ(evals[i], eval_expected)
                        << "BatchOpeningClaim idx=" << idx << ": evaluation " << i << " mismatch";
                }

                ++idx;
            }
        }
    }

    /**
     * @brief Ensures that a 'BatchOpeningClaim' is correct by checking that
     * - all evaluations are correct by recomputing them from each witness polynomial.
     * - commitments are correct by recomputing a commitment from each witness polynomial.
     * - each 'queries' is a subset of 'all_queries' and 'all_queries' is the union of all 'queries'
     * - each 'commitment' of each 'SubClaim' appears only once.
     */
    void verify_batch_opening_claim(std::span<const OpeningClaim<Params>> multi_claims,
                                    std::span<const Polynomial> witnesses)
    {
        const size_t num_claims = multi_claims.size();
        ASSERT_EQ(witnesses.size(), num_claims);

        for (size_t j = 0; j < num_claims; ++j) {
            this->verify_opening_claim(multi_claims[j], witnesses[j]);
        }
    }

    numeric::random::Engine* engine;

    // Per-test-suite set-up.
    // Called before the first test in this test suite.
    // Can be omitted if not needed.
    static void SetUpTestSuite()
    {
        // Avoid reallocating static objects if called in subclasses of FooTest.
        if (commitment_key == nullptr) {
            commitment_key = CreateCommitmentKey<CK>();
        }
        if (verification_key == nullptr) {
            verification_key = CreateVerificationKey<VK>();
        }
    }

    // Per-test-suite tear-down.
    // Called after the last test in this test suite.
    // Can be omitted if not needed.
    static void TearDownTestSuite()
    {
        delete verification_key;
        verification_key = nullptr;
    }

    static typename std::shared_ptr<typename Params::CK> commitment_key;
    static typename Params::VK* verification_key;
};

template <typename Params>
typename std::shared_ptr<typename Params::CK> CommitmentTest<Params>::commitment_key = nullptr;
template <typename Params> typename Params::VK* CommitmentTest<Params>::verification_key = nullptr;

using CommitmentSchemeParams = ::testing::Types<kzg::Params>;
// IMPROVEMENT: reinstate typed-tests for multiple field types, i.e.:
// using CommitmentSchemeParams =
//     ::testing::Types<fake::Params<barretenberg::g1>, fake::Params<grumpkin::g1>, kzg::Params>;

} // namespace honk::pcs