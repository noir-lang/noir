#include "numeric/bitop/get_msb.hpp"
#include "plonk/proof_system/constants.hpp"
#include "proof_system/flavor/flavor.hpp"
#include "prover.hpp"
#include "proof_system/proving_key/proving_key.hpp"
#include "transcript/transcript.hpp"
#include "verifier.hpp"
#include <ecc/curves/bn254/scalar_multiplication/scalar_multiplication.hpp>
#include <gtest/gtest.h>
#include <srs/reference_string/file_reference_string.hpp>
#include <polynomials/polynomial_arithmetic.hpp>
#include <plonk/proof_system/commitment_scheme/kate_commitment_scheme.hpp>
#include <honk/composer/composer_helper/permutation_helper.hpp>

using namespace barretenberg;
using namespace honk;

namespace test_honk_verifier {

template <class FF> class VerifierTests : public testing::Test {
  public:
    // TODO(luke): replace this with an appropriate mock honk manifest
    static transcript::Manifest create_manifest(const size_t num_public_inputs, const size_t num_sumcheck_rounds)
    {
        return honk::StandardHonk::create_unrolled_manifest(num_public_inputs, num_sumcheck_rounds);
    }

    static StandardVerifier generate_verifier(std::shared_ptr<waffle::proving_key> circuit_proving_key)
    {
        std::array<fr*, 8> poly_coefficients;
        poly_coefficients[0] = circuit_proving_key->polynomial_cache.get("q_1_lagrange").get_coefficients();
        poly_coefficients[1] = circuit_proving_key->polynomial_cache.get("q_2_lagrange").get_coefficients();
        poly_coefficients[2] = circuit_proving_key->polynomial_cache.get("q_3_lagrange").get_coefficients();
        poly_coefficients[3] = circuit_proving_key->polynomial_cache.get("q_m_lagrange").get_coefficients();
        poly_coefficients[4] = circuit_proving_key->polynomial_cache.get("q_c_lagrange").get_coefficients();
        poly_coefficients[5] = circuit_proving_key->polynomial_cache.get("sigma_1_lagrange").get_coefficients();
        poly_coefficients[6] = circuit_proving_key->polynomial_cache.get("sigma_2_lagrange").get_coefficients();
        poly_coefficients[7] = circuit_proving_key->polynomial_cache.get("sigma_3_lagrange").get_coefficients();

        std::vector<barretenberg::g1::affine_element> commitments;
        scalar_multiplication::pippenger_runtime_state prover(circuit_proving_key->n);
        commitments.resize(8);

        for (size_t i = 0; i < 8; ++i) {
            commitments[i] = g1::affine_element(
                scalar_multiplication::pippenger(poly_coefficients[i],
                                                 circuit_proving_key->reference_string->get_monomials(),
                                                 circuit_proving_key->n,
                                                 prover));
        }

        auto crs = std::make_shared<waffle::VerifierFileReferenceString>("../srs_db/ignition");
        auto circuit_verification_key = std::make_shared<waffle::verification_key>(
            circuit_proving_key->n, circuit_proving_key->num_public_inputs, crs, circuit_proving_key->composer_type);

        circuit_verification_key->constraint_selectors.insert({ "Q_1", commitments[0] });
        circuit_verification_key->constraint_selectors.insert({ "Q_2", commitments[1] });
        circuit_verification_key->constraint_selectors.insert({ "Q_3", commitments[2] });
        circuit_verification_key->constraint_selectors.insert({ "Q_M", commitments[3] });
        circuit_verification_key->constraint_selectors.insert({ "Q_C", commitments[4] });

        circuit_verification_key->permutation_selectors.insert({ "SIGMA_1", commitments[5] });
        circuit_verification_key->permutation_selectors.insert({ "SIGMA_2", commitments[6] });
        circuit_verification_key->permutation_selectors.insert({ "SIGMA_3", commitments[7] });

        StandardVerifier verifier(circuit_verification_key, create_manifest(0, circuit_proving_key->log_n));

        // TODO(luke): set verifier PCS ala the following:
        // std::unique_ptr<KateCommitmentScheme<standard_settings>> kate_commitment_scheme =
        //     std::make_unique<KateCommitmentScheme<standard_settings>>();
        // verifier.commitment_scheme = std::move(kate_commitment_scheme);

        return verifier;
    }

    // TODO: this example is adapted from a corresponding PlonK verifier test. Needs to be
    // updated further as the Honk PoC comes together.
    static StandardUnrolledProver generate_test_data(const size_t n)
    {
        // Create some constraints that satisfy our arithmetic circuit relation
        // even indices = mul gates, odd incides = add gates

        auto crs = std::make_shared<waffle::FileReferenceString>(n + 1, "../srs_db/ignition");
        std::shared_ptr<waffle::proving_key> proving_key =
            std::make_shared<waffle::proving_key>(n, 0, crs, waffle::STANDARD_HONK);

        polynomial w_l;
        polynomial w_r;
        polynomial w_o;
        polynomial q_l;
        polynomial q_r;
        polynomial q_o;
        polynomial q_c;
        polynomial q_m;

        w_l.resize(n);
        w_r.resize(n);
        w_o.resize(n);
        q_l.resize(n);
        q_r.resize(n);
        q_o.resize(n);
        q_m.resize(n);
        q_c.resize(n);
        fr T0;
        for (size_t i = 0; i < n / 4; ++i) {
            w_l.at(2 * i) = fr::random_element();
            w_r.at(2 * i) = fr::random_element();
            w_o.at(2 * i) = w_l.at(2 * i) * w_r.at(2 * i);
            w_o[2 * i] = w_o[2 * i] + w_l[2 * i];
            w_o[2 * i] = w_o[2 * i] + w_r[2 * i];
            w_o[2 * i] = w_o[2 * i] + fr::one();
            q_l.at(2 * i) = fr::one();
            q_r.at(2 * i) = fr::one();
            q_o.at(2 * i) = fr::neg_one();
            q_c.at(2 * i) = fr::one();
            q_m.at(2 * i) = fr::one();

            w_l.at(2 * i + 1) = fr::random_element();
            w_r.at(2 * i + 1) = fr::random_element();
            w_o.at(2 * i + 1) = fr::random_element();

            T0 = w_l.at(2 * i + 1) + w_r.at(2 * i + 1);
            q_c.at(2 * i + 1) = T0 + w_o.at(2 * i + 1);
            q_c.at(2 * i + 1).self_neg();
            q_l.at(2 * i + 1) = fr::one();
            q_r.at(2 * i + 1) = fr::one();
            q_o.at(2 * i + 1) = fr::one();
            q_m.at(2 * i + 1) = fr::zero();
        }
        size_t shift = n / 2;
        polynomial_arithmetic::copy_polynomial(&w_l.at(0), &w_l.at(shift), shift, shift);
        polynomial_arithmetic::copy_polynomial(&w_r.at(0), &w_r.at(shift), shift, shift);
        polynomial_arithmetic::copy_polynomial(&w_o.at(0), &w_o.at(shift), shift, shift);
        polynomial_arithmetic::copy_polynomial(&q_m.at(0), &q_m.at(shift), shift, shift);
        polynomial_arithmetic::copy_polynomial(&q_l.at(0), &q_l.at(shift), shift, shift);
        polynomial_arithmetic::copy_polynomial(&q_r.at(0), &q_r.at(shift), shift, shift);
        polynomial_arithmetic::copy_polynomial(&q_o.at(0), &q_o.at(shift), shift, shift);
        polynomial_arithmetic::copy_polynomial(&q_c.at(0), &q_c.at(shift), shift, shift);

        std::vector<uint32_t> sigma_1_mapping;
        std::vector<uint32_t> sigma_2_mapping;
        std::vector<uint32_t> sigma_3_mapping;
        // create basic permutation - second half of witness vector is a copy of the first half
        sigma_1_mapping.resize(n);
        sigma_2_mapping.resize(n);
        sigma_3_mapping.resize(n);

        for (size_t i = 0; i < n / 2; ++i) {
            sigma_1_mapping[shift + i] = (uint32_t)i;
            sigma_2_mapping[shift + i] = (uint32_t)i + (1U << 30U);
            sigma_3_mapping[shift + i] = (uint32_t)i + (1U << 31U);
            sigma_1_mapping[i] = (uint32_t)(i + shift);
            sigma_2_mapping[i] = (uint32_t)(i + shift) + (1U << 30U);
            sigma_3_mapping[i] = (uint32_t)(i + shift) + (1U << 31U);
        }
        // make last permutation the same as identity permutation
        // we are setting the permutation in the last 4 gates as identity permutation since
        // we are cutting out 4 roots as of now.
        size_t num_roots_cut_out_of_the_vanishing_polynomial = 4;
        for (uint32_t j = 0; j < num_roots_cut_out_of_the_vanishing_polynomial; ++j) {
            sigma_1_mapping[shift - 1 - j] = (uint32_t)shift - 1 - j;
            sigma_2_mapping[shift - 1 - j] = (uint32_t)shift - 1 - j + (1U << 30U);
            sigma_3_mapping[shift - 1 - j] = (uint32_t)shift - 1 - j + (1U << 31U);
            sigma_1_mapping[n - 1 - j] = (uint32_t)n - 1 - j;
            sigma_2_mapping[n - 1 - j] = (uint32_t)n - 1 - j + (1U << 30U);
            sigma_3_mapping[n - 1 - j] = (uint32_t)n - 1 - j + (1U << 31U);
        }

        polynomial sigma_1(proving_key->n);
        polynomial sigma_2(proving_key->n);
        polynomial sigma_3(proving_key->n);

        // TODO(luke): This is part of the permutation functionality that needs to be updated for honk
        // waffle::compute_permutation_lagrange_base_single<standard_settings>(sigma_1, sigma_1_mapping,
        // proving_key->small_domain); waffle::compute_permutation_lagrange_base_single<standard_settings>(sigma_2,
        // sigma_2_mapping, proving_key->small_domain);
        // waffle::compute_permutation_lagrange_base_single<standard_settings>(sigma_3, sigma_3_mapping,
        // proving_key->small_domain);

        polynomial sigma_1_lagrange_base(sigma_1, proving_key->n);
        polynomial sigma_2_lagrange_base(sigma_2, proving_key->n);
        polynomial sigma_3_lagrange_base(sigma_3, proving_key->n);

        proving_key->polynomial_cache.put("sigma_1_lagrange", std::move(sigma_1_lagrange_base));
        proving_key->polynomial_cache.put("sigma_2_lagrange", std::move(sigma_2_lagrange_base));
        proving_key->polynomial_cache.put("sigma_3_lagrange", std::move(sigma_3_lagrange_base));

        honk::compute_standard_honk_id_polynomials<3>(proving_key);
        honk::compute_first_and_last_lagrange_polynomials(proving_key);

        proving_key->polynomial_cache.put("w_1_lagrange", std::move(w_l));
        proving_key->polynomial_cache.put("w_2_lagrange", std::move(w_r));
        proving_key->polynomial_cache.put("w_3_lagrange", std::move(w_o));

        proving_key->polynomial_cache.put("q_1_lagrange", std::move(q_l));
        proving_key->polynomial_cache.put("q_2_lagrange", std::move(q_r));
        proving_key->polynomial_cache.put("q_3_lagrange", std::move(q_o));
        proving_key->polynomial_cache.put("q_m_lagrange", std::move(q_m));
        proving_key->polynomial_cache.put("q_c_lagrange", std::move(q_c));

        // TODO(Cody): This should be more generic
        StandardUnrolledProver prover = StandardUnrolledProver(proving_key, create_manifest(0, proving_key->log_n));

        std::unique_ptr<pcs::kzg::CommitmentKey> kate_commitment_key =
            std::make_unique<pcs::kzg::CommitmentKey>(proving_key->n, "../srs_db/ignition");

        prover.commitment_key = std::move(kate_commitment_key);

        return prover;
    }
};

using FieldTypes = testing::Types<barretenberg::fr>;
TYPED_TEST_SUITE(VerifierTests, FieldTypes);

// This test is modeled after a corresponding test for the Plonk Verifier. As is the case there, this test relies on
// valid proof construction which makes the scope quite large. Not really a unit test but a nice test nonetheless.
TYPED_TEST(VerifierTests, VerifyArithmeticProofSmall)
{
    GTEST_SKIP() << "It's good to have a standalone test, but for now we just rely on composer tests.";
    size_t n = 8;

    StandardUnrolledProver prover = TestFixture::generate_test_data(n);

    StandardVerifier verifier = TestFixture::generate_verifier(prover.proving_key);

    // construct proof
    waffle::plonk_proof proof = prover.construct_proof();

    // verify proof
    bool result = verifier.verify_proof(proof);

    EXPECT_EQ(result, true);
}

} // namespace test_honk_verifier