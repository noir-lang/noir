#pragma once

#include <cstddef>
#include <gtest/gtest.h>

#include <concepts>
#include <algorithm>
#include <memory>
#include <string_view>

#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/srs/reference_string/file_reference_string.hpp"
#include "barretenberg/ecc/curves/bn254/g1.hpp"

#include "../oracle/oracle.hpp"
#include "../../transcript/transcript_wrappers.hpp"

#include "claim.hpp"
#include "commitment_key.hpp"

namespace proof_system::honk::pcs {
namespace {
constexpr std::string_view kzg_srs_path = "../srs_db/ignition";
}

template <class CK> inline std::shared_ptr<CK> CreateCommitmentKey();

template <> inline std::shared_ptr<kzg::CommitmentKey> CreateCommitmentKey<kzg::CommitmentKey>()
{
    const size_t n = 128;
    return std::make_shared<kzg::CommitmentKey>(n, kzg_srs_path);
}
// For IPA
template <> inline std::shared_ptr<ipa::CommitmentKey> CreateCommitmentKey<ipa::CommitmentKey>()
{
    const size_t n = 128;
    return std::make_shared<ipa::CommitmentKey>(n, kzg_srs_path);
}

template <typename CK> inline std::shared_ptr<CK> CreateCommitmentKey()
// requires std::default_initializable<CK>
{
    return std::make_shared<CK>();
}

template <class VK> inline std::shared_ptr<VK> CreateVerificationKey();

template <> inline std::shared_ptr<kzg::VerificationKey> CreateVerificationKey<kzg::VerificationKey>()
{
    return std::make_shared<kzg::VerificationKey>(kzg_srs_path);
}
// For IPA
template <> inline std::shared_ptr<ipa::VerificationKey> CreateVerificationKey<ipa::VerificationKey>()
{
    const size_t n = 128;
    return std::make_shared<ipa::VerificationKey>(n, kzg_srs_path);
}
template <typename VK> inline std::shared_ptr<VK> CreateVerificationKey()
// requires std::default_initializable<VK>
{
    return std::make_shared<VK>();
}
template <typename Params> class CommitmentTest : public ::testing::Test {
    using CK = typename Params::CK;
    using VK = typename Params::VK;

    using Fr = typename Params::Fr;
    using CommitmentAffine = typename Params::C;
    using Polynomial = typename Params::Polynomial;
    using Transcript = transcript::StandardTranscript;

  public:
    CommitmentTest()
        : engine{ &numeric::random::get_debug_engine() }
    {}

    std::shared_ptr<CK> ck() { return commitment_key; }
    std::shared_ptr<VK> vk() { return verification_key; }

    CommitmentAffine commit(const Polynomial& polynomial) { return commitment_key->commit(polynomial); }

    Polynomial random_polynomial(const size_t n)
    {
        Polynomial p(n);
        for (size_t i = 0; i < n; ++i) {
            p[i] = Fr::random_element(engine);
        }
        return p;
    }

    Fr random_element() { return Fr::random_element(engine); }

    OpeningPair<Params> random_eval(const Polynomial& polynomial)
    {
        Fr x{ random_element() };
        Fr y{ polynomial.evaluate(x) };
        return { x, y };
    }

    std::pair<OpeningClaim<Params>, Polynomial> random_claim(const size_t n)
    {
        auto polynomial = random_polynomial(n);
        auto opening_pair = random_eval(polynomial);
        auto commitment = commit(polynomial);
        auto opening_claim = OpeningClaim<Params>{ opening_pair, commitment };
        return { opening_claim, polynomial };
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
        auto& commitment = claim.commitment;
        auto& [x, y] = claim.opening_pair;
        Fr y_expected = witness.evaluate(x);
        EXPECT_EQ(y, y_expected) << "OpeningClaim: evaluations mismatch";
        CommitmentAffine commitment_expected = commit(witness);
        EXPECT_EQ(commitment, commitment_expected) << "OpeningClaim: commitment mismatch";
    }

    void verify_opening_pair(const OpeningPair<Params>& opening_pair, const Polynomial& witness)
    {
        auto& [x, y] = opening_pair;
        Fr y_expected = witness.evaluate(x);
        EXPECT_EQ(y, y_expected) << "OpeningPair: evaluations mismatch";
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

    /**
     * @brief Ensures that a set of opening pairs is correct by checking that evaluations are
     * correct by recomputing them from each witness polynomial.
     */
    void verify_batch_opening_pair(std::span<const OpeningPair<Params>> opening_pairs,
                                   std::span<const Polynomial> witnesses)
    {
        const size_t num_pairs = opening_pairs.size();
        ASSERT_EQ(witnesses.size(), num_pairs);

        for (size_t j = 0; j < num_pairs; ++j) {
            this->verify_opening_pair(opening_pairs[j], witnesses[j]);
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
    static void TearDownTestSuite() {}

    static typename std::shared_ptr<typename Params::CK> commitment_key;
    static typename std::shared_ptr<typename Params::VK> verification_key;
};

template <typename Params>
typename std::shared_ptr<typename Params::CK> CommitmentTest<Params>::commitment_key = nullptr;
template <typename Params>
typename std::shared_ptr<typename Params::VK> CommitmentTest<Params>::verification_key = nullptr;

using CommitmentSchemeParams = ::testing::Types<kzg::Params>;
using IpaCommitmentSchemeParams = ::testing::Types<ipa::Params>;
// IMPROVEMENT: reinstate typed-tests for multiple field types, i.e.:
// using CommitmentSchemeParams =
//     ::testing::Types<fake::Params<barretenberg::g1>, fake::Params<grumpkin::g1>, kzg::Params>;

} // namespace proof_system::honk::pcs
